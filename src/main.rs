mod tgv;
use leptos::{html::Input, logging::log, prelude::*, task::spawn_local};
use console_error_panic_hook;
use web_sys::{js_sys, wasm_bindgen::JsCast, HtmlInputElement};
use image::{DynamicImage, ImageBuffer, ImageFormat, RgbImage, GrayImage};
use std::io::Cursor;
use base64::{engine::general_purpose, Engine as _};
// use wasm_bindgen::prelude::*;
use ndarray::{Array2, Array3, s};
use nshare::{self, AsNdarray3};


async fn convert_image_input_to_base_64(input: Option<HtmlInputElement>) -> Result<String, String> {
    let input = input.ok_or("No input element found")?;
    let files = input.files().ok_or("No files selected")?;
    let file = files.get(0).ok_or("No file found")?;

    // Read file as ArrayBuffer
    let array_buffer_promise = file.array_buffer();
    let array_buffer = wasm_bindgen_futures::JsFuture::from(array_buffer_promise)
        .await
        .map_err(|e| format!("Failed to read file: {:?}", e))?;

    // Convert to Uint8Array and then to Vec<u8>
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let buffer_vec = uint8_array.to_vec();

    // Decode the image
    let img = image::load_from_memory(&buffer_vec)
        .map_err(|e| format!("Failed to decode image: {:?}", e))?;
    let img: RgbImage = img.into_rgb8();

    // Convert original image to base64 for display
    let mut original_buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut original_buffer), ImageFormat::Png)
        .map_err(|e| format!("Failed to encode original image: {:?}", e))?;
    let original_base64 = general_purpose::STANDARD.encode(&original_buffer);
    Ok(format!("data:image/png;base64,{}", original_base64))
}


async fn process_image(input: Option<HtmlInputElement>, tgv_lam: f32) -> Result<(String, String), String> {
    let input = input.ok_or("No input element found")?;
    let files = input.files().ok_or("No files selected")?;
    let file = files.get(0).ok_or("No file found")?;

    // Read file as ArrayBuffer
    let array_buffer_promise = file.array_buffer();
    let array_buffer = wasm_bindgen_futures::JsFuture::from(array_buffer_promise)
        .await
        .map_err(|e| format!("Failed to read file: {:?}", e))?;

    // Convert to Uint8Array and then to Vec<u8>
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let buffer_vec = uint8_array.to_vec();

    // Decode the image
    let img = image::load_from_memory(&buffer_vec)
        .map_err(|e| format!("Failed to decode image: {:?}", e))?;
    let img: RgbImage = img.into_rgb8();

    // Convert original image to base64 for display
    let mut original_buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut original_buffer), ImageFormat::Png)
        .map_err(|e| format!("Failed to encode original image: {:?}", e))?;
    let original_base64 = general_purpose::STANDARD.encode(&original_buffer);
    let original_data_url = format!("data:image/png;base64,{}", original_base64);

    // Process the image with TGV denoising
    // let rgb_img: RgbImage = img.to_rgb8();
    let img = img.as_ndarray3();
    let img = img.permuted_axes([1, 2, 0]);
    // println!("img shape is {:?}", img.shape());
    // println!("img min is {:?}", img.into_iter().min());
    // println!("img max is {:?}", img.into_iter().max());
    let img: Array3<f32> = img.map(|x| *x as f32);
    let grayscale_img: Array2<f32> = (&img.slice(s![.., .., 0]) + &img.slice(s![.., .., 1]) + &img.slice(s![.., .., 2])) / 3.0;

    // println!("grayscale_img min is {:?}", (&grayscale_img).into_iter().reduce(|a, b| if a < b { a } else { b }));
    // println!("grayscale_img max is {:?}", (&grayscale_img).into_iter().reduce(|a, b| if a > b { a } else { b }));

    let denoised_img = tgv::tgv_denoise(&grayscale_img.view(), tgv_lam, 2.0, 1.0, 0.125, 0.125, 300);
    // WebAssembly does not allow for parallelization directly using rayon. Needs special handling
    let denoised_img = denoised_img.map(|x| *x as u8);
    let denoised_img = GrayImage::from_raw(img.shape()[0] as u32, img.shape()[1] as u32, denoised_img.into_iter().collect()).unwrap();

    // Convert processed image to base64 for display
    let mut processed_buffer = Vec::new();
    denoised_img.write_to(&mut Cursor::new(&mut processed_buffer), ImageFormat::Png)
        .map_err(|e| format!("Failed to encode processed image: {:?}", e))?;
    let processed_base64 = general_purpose::STANDARD.encode(&processed_buffer);
    let processed_data_url = format!("data:image/png;base64,{}", processed_base64);

    Ok((original_data_url, processed_data_url))
}


#[component]
fn SyncedControl(tgv_lam: ReadSignal<f32>, tgv_lam_setter: WriteSignal<f32>) -> impl IntoView {
    // Reactive state for our parameter (range 0.0–1.0)

    view! {
      <div style="display: flex; align-items: center; gap: 8px;">
        // Slider: -1e-12 to 1e3 but in log scale
        <input
          type="range"
          min="-12"
          max="3"
          step="0.1"
          // Bind slider thumb to param
          prop:value=move || tgv_lam.get().to_string()  // Let the slider cursor go from min to max
          // On input, parse and update `param`
          on:input=move |ev| {
            let v = event_target_value(&ev)
                      .parse::<f32>()
                    //   .map(|x| x.exp())
                      .unwrap_or(tgv_lam.get());
                    // tgv_lam.set(v);
            tgv_lam_setter.set(v);
          }
        />

        // Number input: shows same param but in linear scale
        <input
          type="number"
          step="1e-3"
          min="1e-12"
          max="1e3"
          prop:value=move || tgv_lam.get().exp().to_string()  // Show hte displayed value in log scale
        //   On input, parse and update `param`
          on:input=move |ev| {
            let v = event_target_value(&ev)
                      .parse::<f32>()
                      .map(|x| x.ln())
                      .unwrap_or(tgv_lam.get());  // Int inputted, store it as log scale
            // param_set.set(v);
            tgv_lam_setter.set(v);
          }
          style="width: 4em;"
        />
      </div>
    }
}


#[component]
fn App() -> impl IntoView {
    let file_input: NodeRef<Input> = NodeRef::new();
    let (original_img_src, set_original_img_src) = signal(String::new());
    let (processed_img_src, set_processed_img_src) = signal(String::new());
    let (is_processing, set_is_processing) = signal(false);
    let (error_message, set_error_message) = signal(String::new());
    let (tgv_lam, set_tgv_lam) = signal(0.5 as f32);


    // Use spawn_local directly in the click handler instead of Action
    let on_process = move |_| {
        let file_input_clone = file_input.get();

        // Clone the setters to avoid capturing references
        let set_original_img_src = set_original_img_src.clone();
        let set_processed_img_src = set_processed_img_src.clone();
        let set_is_processing = set_is_processing.clone();
        let set_error_message = set_error_message.clone();
        // let tgv_lam = 

        set_is_processing.set(true);
        set_error_message.set(String::new());

        spawn_local(async move {
            match process_image(file_input_clone, tgv_lam.get().exp()).await {
                Ok((original, processed)) => {
                    set_original_img_src.set(original);
                    set_processed_img_src.set(processed);
                    set_is_processing.set(false);
                },
                Err(err) => {
                    set_error_message.set(err);
                    set_is_processing.set(false);
                }
            }
        });
    };

    let update_image = move |_| {
        spawn_local(async move {
                let input_element = file_input.get();
                set_original_img_src.set(convert_image_input_to_base_64(input_element).await.unwrap_or_default());
            }
        )
    };

    view! {
        <div class="container">
            <h1>"TGV Image Denoising"</h1>

            <div class="upload-section">
                <input 
                    type="file" 
                    accept="image/*" 
                    node_ref=file_input
                    // on:
                    on:change=update_image
                />
                <button
                    on:click=on_process
                    disabled=is_processing
                >
                // "Process Image"
                    {move || if is_processing.get() { "Processing..." } else { "Process Image" }}
                </button>
            </div>
            <SyncedControl tgv_lam=tgv_lam tgv_lam_setter=set_tgv_lam />

            // {move || error_message().as_str().is_empty().then(|| view! {
            //     <div class="error-message">{error_message}</div>
            // })}

            <div class="image-container">
                <Show when=move || !original_img_src.get().is_empty()>
                    <div class="image-box">
                        <h2>"Original Image"</h2>
                        <img src=original_img_src alt="Original Image" />
                    </div>
                </Show>

                <Show when=move || !processed_img_src.get().is_empty()>
                    <div class="image-box">
                        <h2>"Denoised Image, " {move || format!("lambda = {:.3}", tgv_lam.get().exp())}</h2>
                        <img src=processed_img_src alt="Denoised Image" />
                    </div>
                </Show>
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App)
}
