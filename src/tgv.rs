// Library version of main.rs

use ndarray::{s, Axis, NewAxis, Array, Array1, Array2, Array3, ArrayView, ArrayView1, ArrayView2, ArrayView3};
// use ndarray::linalg;
// use ndarray_linalg::Norm;
// use ndarray_rand::RandomExt;
// use ndarray_rand::rand_distr::Normal;
use rayon::prelude::*;

fn roll1d(a: &ArrayView1<f32>, roll_amount: i32) -> Array1<f32> {
    
    return ndarray::concatenate![
        Axis(0), 
        a.slice(s![-roll_amount..]), 
        a.slice(s![..-roll_amount])]
}


fn roll2d(a: &ArrayView2<f32>, axis: usize, roll_amount: i32) -> Array2<f32> {
    assert!(roll_amount.abs() > 0);
    if axis == 0 {
        return ndarray::concatenate![Axis(0), a.slice(s![-roll_amount.., ..]), a.slice(s![..-roll_amount, ..])]
    } else if axis == 1 {
        ndarray::concatenate![Axis(1), a.slice(s![.., -roll_amount..]), a.slice(s![.., ..-roll_amount,])]
    } else {
        return a.to_owned()
    }
}

fn gradient(u: &ArrayView2<f32>) -> Array3<f32> {
    let grad_x = roll2d(&u.view(), 1, -1) - u;
    let grad_y = roll2d(&u.view(), 0, -1) - u;

    ndarray::stack![Axis(2), grad_x, grad_y]
}

fn divergence(p: &ArrayView3<f32>) -> Array2<f32> {
    let first_term = p.slice(s![.., .., 0]).to_owned() 
        - roll2d(&p.slice(s![.., .., 0]), 1, 1);
    let second_term = p.slice(s![.., .., 1]).to_owned() 
    - roll2d(&p.slice(s![.., .., 1]), 0, 1);
    -(first_term + second_term)
}

fn sym_gradient(w: &ArrayView3<f32>) -> Array3<f32> {
    // First diagonal: ∂x w_0
    let first_diagonal = roll2d(&w.slice(s![.., .., 0]), 1, -1) 
        - w.slice(s![.., .., 0]);
    // Second diagonal: ∂y w_1
    let second_diagonal = roll2d(&w.slice(s![.., .., 1]), 0, -1) 
        - w.slice(s![.., .., 1]);
    // Off-diagonals: 0.5*(∂y w_0 + ∂x w_1)
    let tmp1 = roll2d(&w.slice(s![.., .., 0]), 0, -1) 
        - w.slice(s![.., .., 0]);
    let tmp2 = roll2d(&w.slice(s![.., .., 1]), 1, -1) 
        - w.slice(s![.., .., 1]);
    let off_diagonals = 0.5 * (tmp1 + tmp2);

    ndarray::stack![Axis(2), first_diagonal, second_diagonal, off_diagonals]
}

fn sym_divergence(q: &ArrayView3<f32>) -> Array3<f32> {
    // First component: ∂x q_0 - ∂y q_2
    let first_term = -(q.slice(s![.., .., 0]).to_owned() 
        - roll2d(&q.slice(s![.., .., 0]), 1, 1));
    let second_term = -0.5 * (q.slice(s![.., .., 2]).to_owned() 
        - roll2d(&q.slice(s![.., .., 2]), 0, 1));
    let first_component = first_term + second_term;
    // Second component: ∂y q_1 - ∂x q_2
    let first_term = -(q.slice(s![.., .., 1]).to_owned() 
        - roll2d(&q.slice(s![.., .., 1]), 0, 1));
    let second_term = -0.5 * (q.slice(s![.., .., 2]).to_owned() 
        - roll2d(&q.slice(s![.., .., 2]), 1, 1));
    let second_component = first_term + second_term;
    ndarray::stack![Axis(2), first_component, second_component]
}

fn proj_p(p: &ArrayView3<f32>, alpha1: &f32) -> Array3<f32> {
    let norm = (p.slice(s![.., .., 0]).map(|x| x.powi(2)) 
        + p.slice(s![.., .., 1]).map(|x| x.powi(2)))
        .sqrt();
    let factor = norm.map(|x| if (x / alpha1) > 1. { x / alpha1 } else { 1. });
    let mut p_proj = p.to_owned();
    let mut slice1 = p_proj.slice_mut(s![.., .., 0]);
    slice1 /= &factor;
    let mut slice2 = p_proj.slice_mut(s![.., .., 1]);
    slice2 /= &factor;
    p_proj
}

fn proj_q(q: &ArrayView3<f32>, alpha0: &f32) -> Array3<f32> {
    let norm = (q.slice(s![.., .., 0]).map(|x| x.powi(2)) 
        + q.slice(s![.., .., 1]).map(|x| x.powi(2)) 
        + q.slice(s![.., .., 2]).map(|x| x.powi(2)))
        .sqrt();
    let factor = norm.map(|x| if (x / alpha0) > 1. { x / alpha0 } else { 1. });
    let mut q_proj = q.to_owned();
    let mut slice1 = q_proj.slice_mut(s![.., .., 0]);
    slice1 /= &factor;
    let mut slice2 = q_proj.slice_mut(s![.., .., 1]);
    slice2 /= &factor;
    let mut slice3 = q_proj.slice_mut(s![.., .., 2]);
    slice3 /= &factor;
    q_proj
}


pub fn tgv_denoise(u0: &ArrayView2<f32>, lam: f32, alpha0: f32, alpha1: f32, tau: f32, sigma: f32, n_iter: i32) -> Array2<f32> {
    let mut u = u0.to_owned();
    let mut w = Array3::<f32>::zeros((u0.shape()[0], u0.shape()[1], 2));
    let mut p = Array3::<f32>::zeros((u0.shape()[0], u0.shape()[1], 2));
    let mut q = Array3::<f32>::zeros((u0.shape()[0], u0.shape()[1], 3));

    let mut u_bar = u.clone();
    let mut w_bar = w.clone();
    let mut u_old;
    let mut w_old;

    for i in 0..n_iter {
        let grad_u_bar = gradient(&u_bar.view());
        p = &p + (&grad_u_bar - &w_bar) * sigma;
        p = proj_p(&p.view(), &(alpha1 * lam));

        let q_bar = sym_gradient(&w_bar.view());
        q = &q + &q_bar * sigma;
        q = proj_q(&q.view(), &(alpha0 * lam));

        u_old = u.clone();
        w_old = w.clone();

        u = u - tau * divergence(&p.view());
        u = u + u0 * tau;
        u = u / (1. + tau);

        w = w - tau * (-&p + sym_divergence(&q.view()));

        u_bar = 2. * &u - &u_old;
        w_bar = 2. * &w - &w_old;

        // if i % 50 == 0 {
        //     let primal_res = (&u - u_old).norm();
        //     println!("Iteration {:?}, primal change = {:?}", i, primal_res);
        // }
    }
    u
}

pub fn parallel_tgv_denoise(u0: &ArrayView2<f32>, lam: f32, alpha0: f32, alpha1: f32, tau: f32, sigma: f32, n_iter: i32) -> Array2<f32> {
    // Split the image into patches
    let patch_size = 32;
    let num_patches_x = u0.shape()[0] / patch_size;
    let num_patches_y = u0.shape()[1] / patch_size;

    // Create a vector to store the denoised patches
    // let mut patches: Array3<f32> = Array3::<f32>::zeros((num_patches_x * num_patches_y, patch_size, patch_size));
    let mut patches = Vec::new();
    for i in 0..num_patches_x {
        for j in 0..num_patches_y {
            // let mut patch_slice = patches.slice_mut(s![i * num_patches_y + j, .., ..]);
            // patch_slice.assign(&u0.slice(s![(i * patch_size)..((i + 1) * patch_size), (j * patch_size)..((j + 1) * patch_size)]));
            patches.push(u0.slice(s![(i * patch_size)..((i + 1) * patch_size), (j * patch_size)..((j + 1) * patch_size)]).to_owned());
        }
    }

    let denoised_patches: Vec<Array2<f32>> = patches.par_iter().map(|patch| {
        tgv_denoise(&patch.view(), lam, alpha0, alpha1, tau, sigma, n_iter)
    }).collect();

    // Create a new image to store the denoised patches
    let mut denoised_img = Array2::<f32>::zeros((u0.shape()[0], u0.shape()[1]));
    for i in 0..num_patches_x {
        for j in 0..num_patches_y {
            denoised_img.slice_mut(s![(i * patch_size)..((i + 1) * patch_size), (j * patch_size)..((j + 1) * patch_size)]).assign(&denoised_patches[i * num_patches_y + j]);
        }
    }

    return denoised_img;
}

