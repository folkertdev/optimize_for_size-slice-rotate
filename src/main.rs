#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

use core::{hint::black_box, mem::MaybeUninit};

#[cfg_attr(not(test), cortex_m_rt::entry)]
fn main() -> ! {
    #[cfg(feature = "left-std")]
    status_quo_left(black_box(&mut [1, 2, 3]), black_box(1));
    #[cfg(feature = "right-std")]
    status_quo_right(black_box(&mut [1, 2, 3]), black_box(1));

    #[cfg(feature = "left-size")]
    left_size(black_box(&mut [1, 2, 3]), black_box(1));
    #[cfg(feature = "right-size")]
    right_size(black_box(&mut [1, 2, 3]), black_box(1));

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(feature = "left-std")]
fn status_quo_left(slice: &mut [u8], mid: usize) {
    slice.rotate_left(mid);
}

#[cfg(feature = "right-std")]
fn status_quo_right(slice: &mut [u8], mid: usize) {
    slice.rotate_right(mid);
}

#[cfg(feature = "left-size")]
fn left_size(slice: &mut [u8], mid: usize) {
    assert!(mid <= slice.len());
    let k = slice.len() - mid;
    let p = slice.as_mut_ptr();

    // SAFETY: The range `[p.add(mid) - mid, p.add(mid) + k)` is trivially
    // valid for reading and writing, as required by `ptr_rotate`.
    unsafe {
        ptr_rotate_opt_for_size(mid, p.add(mid), k);
    }
}

#[cfg(feature = "right-size")]
fn right_size(slice: &mut [u8], k: usize) {
    assert!(k <= slice.len());
    let mid = slice.len() - k;
    let p = slice.as_mut_ptr();

    // SAFETY: The range `[p.add(mid) - mid, p.add(mid) + k)` is trivially
    // valid for reading and writing, as required by `ptr_rotate`.
    unsafe {
        ptr_rotate_opt_for_size(mid, p.add(mid), k);
    }
}

pub unsafe fn ptr_rotate_opt_for_size<T>(mut left: usize, mut mid: *mut T, mut right: usize) {
    use core::{cmp, mem, ptr};
    type BufType = [usize; 32];
    if core::mem::size_of::<T>() == 0 {
        return;
    }
    loop {
        // N.B. the below algorithms can fail if these cases are not checked
        if (right == 0) || (left == 0) {
            return;
        }
        if !cfg!(any(feature = "left-size", feature = "right-size")) && (left + right < 24)
            || (mem::size_of::<T>() > mem::size_of::<[usize; 4]>())
        {
            // Algorithm 1
            // Microbenchmarks indicate that the average performance for random shifts is better all
            // the way until about `left + right == 32`, but the worst case performance breaks even
            // around 16. 24 was chosen as middle ground. If the size of `T` is larger than 4
            // `usize`s, this algorithm also outperforms other algorithms.
            // SAFETY: callers must ensure `mid - left` is valid for reading and writing.
            let x = unsafe { mid.sub(left) };
            // beginning of first round
            // SAFETY: see previous comment.
            let mut tmp: T = unsafe { x.read() };
            let mut i = right;
            // `gcd` can be found before hand by calculating `gcd(left + right, right)`,
            // but it is faster to do one loop which calculates the gcd as a side effect, then
            // doing the rest of the chunk
            let mut gcd = right;
            // benchmarks reveal that it is faster to swap temporaries all the way through instead
            // of reading one temporary once, copying backwards, and then writing that temporary at
            // the very end. This is possibly due to the fact that swapping or replacing temporaries
            // uses only one memory address in the loop instead of needing to manage two.
            loop {
                // [long-safety-expl]
                // SAFETY: callers must ensure `[left, left+mid+right)` are all valid for reading and
                // writing.
                //
                // - `i` start with `right` so `mid-left <= x+i = x+right = mid-left+right < mid+right`
                // - `i <= left+right-1` is always true
                //   - if `i < left`, `right` is added so `i < left+right` and on the next
                //     iteration `left` is removed from `i` so it doesn't go further
                //   - if `i >= left`, `left` is removed immediately and so it doesn't go further.
                // - overflows cannot happen for `i` since the function's safety contract ask for
                //   `mid+right-1 = x+left+right` to be valid for writing
                // - underflows cannot happen because `i` must be bigger or equal to `left` for
                //   a subtraction of `left` to happen.
                //
                // So `x+i` is valid for reading and writing if the caller respected the contract
                tmp = unsafe { x.add(i).replace(tmp) };
                // instead of incrementing `i` and then checking if it is outside the bounds, we
                // check if `i` will go outside the bounds on the next increment. This prevents
                // any wrapping of pointers or `usize`.
                if i >= left {
                    i -= left;
                    if i == 0 {
                        // end of first round
                        // SAFETY: tmp has been read from a valid source and x is valid for writing
                        // according to the caller.
                        unsafe { x.write(tmp) };
                        break;
                    }
                    // this conditional must be here if `left + right >= 15`
                    if i < gcd {
                        gcd = i;
                    }
                } else {
                    i += right;
                }
            }
            // finish the chunk with more rounds
            for start in 1..gcd {
                // SAFETY: `gcd` is at most equal to `right` so all values in `1..gcd` are valid for
                // reading and writing as per the function's safety contract, see [long-safety-expl]
                // above
                tmp = unsafe { x.add(start).read() };
                // [safety-expl-addition]
                //
                // Here `start < gcd` so `start < right` so `i < right+right`: `right` being the
                // greatest common divisor of `(left+right, right)` means that `left = right` so
                // `i < left+right` so `x+i = mid-left+i` is always valid for reading and writing
                // according to the function's safety contract.
                i = start + right;
                loop {
                    // SAFETY: see [long-safety-expl] and [safety-expl-addition]
                    tmp = unsafe { x.add(i).replace(tmp) };
                    if i >= left {
                        i -= left;
                        if i == start {
                            // SAFETY: see [long-safety-expl] and [safety-expl-addition]
                            unsafe { x.add(start).write(tmp) };
                            break;
                        }
                    } else {
                        i += right;
                    }
                }
            }
            return;
        // `T` is not a zero-sized type, so it's okay to divide by its size.
        } else if !cfg!(any(feature = "left-size", feature = "right-size"))
            && cmp::min(left, right) <= mem::size_of::<BufType>() / mem::size_of::<T>()
        {
            // Algorithm 2
            // The `[T; 0]` here is to ensure this is appropriately aligned for T
            let mut rawarray = MaybeUninit::<(BufType, [T; 0])>::uninit();
            let buf = rawarray.as_mut_ptr() as *mut T;
            // SAFETY: `mid-left <= mid-left+right < mid+right`
            let dim = unsafe { mid.sub(left).add(right) };
            if left <= right {
                // SAFETY:
                //
                // 1) The `else if` condition about the sizes ensures `[mid-left; left]` will fit in
                //    `buf` without overflow and `buf` was created just above and so cannot be
                //    overlapped with any value of `[mid-left; left]`
                // 2) [mid-left, mid+right) are all valid for reading and writing and we don't care
                //    about overlaps here.
                // 3) The `if` condition about `left <= right` ensures writing `left` elements to
                //    `dim = mid-left+right` is valid because:
                //    - `buf` is valid and `left` elements were written in it in 1)
                //    - `dim+left = mid-left+right+left = mid+right` and we write `[dim, dim+left)`
                unsafe {
                    // 1)
                    ptr::copy_nonoverlapping(mid.sub(left), buf, left);
                    // 2)
                    ptr::copy(mid, mid.sub(left), right);
                    // 3)
                    ptr::copy_nonoverlapping(buf, dim, left);
                }
            } else {
                // SAFETY: same reasoning as above but with `left` and `right` reversed
                unsafe {
                    ptr::copy_nonoverlapping(mid, buf, right);
                    ptr::copy(mid.sub(left), dim, left);
                    ptr::copy_nonoverlapping(buf, mid.sub(left), right);
                }
            }
            return;
        } else if left >= right {
            // Algorithm 3
            // There is an alternate way of swapping that involves finding where the last swap
            // of this algorithm would be, and swapping using that last chunk instead of swapping
            // adjacent chunks like this algorithm is doing, but this way is still faster.
            loop {
                // SAFETY:
                // `left >= right` so `[mid-right, mid+right)` is valid for reading and writing
                // Subtracting `right` from `mid` each turn is counterbalanced by the addition and
                // check after it.
                unsafe {
                    ptr::swap_nonoverlapping(mid.sub(right), mid, right);
                    mid = mid.sub(right);
                }
                left -= right;
                if left < right {
                    break;
                }
            }
        } else {
            // Algorithm 3, `left < right`
            loop {
                // SAFETY: `[mid-left, mid+left)` is valid for reading and writing because
                // `left < right` so `mid+left < mid+right`.
                // Adding `left` to `mid` each turn is counterbalanced by the subtraction and check
                // after it.
                unsafe {
                    ptr::swap_nonoverlapping(mid.sub(left), mid, left);
                    mid = mid.add(left);
                }
                right -= left;
                if right < left {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn runner() {
        ::quickcheck::quickcheck(test as fn(_, _) -> _);

        fn test(data: Vec<u8>, mid: usize) -> bool {
            if data.is_empty() {
                return true;
            }

            let mid = mid % data.len();

            #[cfg(feature = "left-size")]
            {
                let mut a = data.clone();
                let mut b = data.clone();
                assert_eq!(a.rotate_left(mid), left_size(&mut b, mid));
            }

            #[cfg(feature = "right-size")]
            {
                let mut a = data.clone();
                let mut b = data.clone();
                assert_eq!(a.rotate_right(mid), right_size(&mut b, mid));
            }

            true
        }
    }
}
