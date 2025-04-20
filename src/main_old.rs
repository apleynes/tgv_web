mod tgv;
use leptos::{html::Input, logging::log, prelude::*, task::spawn_local};
use console_error_panic_hook;
use web_sys::{js_sys, HtmlInputElement};
use image::{buffer, codecs::png::PngDecoder, ImageBuffer, ImageDecoder, ImageReader, RgbImage};
// use image::PngDecoder;
use std::io::{Cursor, Write};

// https://github.com/leptos-rs/leptos/discussions/3134

async fn PrintFileContent(input: Option<HtmlInputElement>) -> RgbImage {
    log!("Input: {:?}", input);
    let value = input.unwrap().files();
    let value_unwrapped = value.unwrap();
    let file = value_unwrapped.get(0).unwrap();

    // Use arrayBuffer() instead of text() for binary files
    let array_buffer_promise = file.array_buffer();
    let array_buffer = wasm_bindgen_futures::JsFuture::from(array_buffer_promise).await.unwrap();

    // Convert the array buffer to a Uint8Array
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);

    // Convert to a Rust Vec<u8>
    let buffer_vec = uint8_array.to_vec();
    log!("Buffer vec length: {}", buffer_vec.len());

    // Now use the binary data with image crate
    let img = ImageReader::with_format(
        Cursor::new(buffer_vec),
        image::ImageFormat::Png
    );

    log!("Image reader format: {:?}", img.format());
    let img = img.decode().expect("Failed to decode image");
    // log!("Image dimensions: {:?}", img.dimensions());
    // img.save("img.png").expect("Failed to save image");

    img.to_rgb8()

    // Further processing with the image...
}

// async fn PrintFileContent(input: Option<HtmlInputElement>) {
//     log!("Input: {:?}", input);
//     let value = input.unwrap().files();
//     log!("files: {:?}", value);
//     let value_unwrapped = value.unwrap();
//     println!("Files: {:?}", value_unwrapped);
//     log!("Files: {:?}", value_unwrapped);
//     let get_file = value_unwrapped.get(0);
//     log!("File option: {:?}", get_file);
//     let file_text = get_file.unwrap().text();
//     log!("File text: {:?}", file_text);
//     let result = wasm_bindgen_futures::JsFuture::from(file_text).await;
//     log!("Result: {:?}", result);
//     let buffer_data = result.unwrap();
//     log!("Buffer data: {:?}", buffer_data);
//     // let buffer = js_sys::Uint8Array::new(&buffer_data);
//     log!("Buffer: {:?}", buffer_data.as_string().unwrap());
//     let buffer_vec = buffer_data.as_string().unwrap().as_bytes().to_vec();
//     log!("Buffer vec: {:?}", buffer_vec);

//     let buffer_string = buffer_data.as_string().unwrap();
//     let buffer_bytes = buffer_string.as_bytes();
//     log!("Buffer bytes: {:?}", buffer_bytes);

//     // std::fs::write("img_from_raw_buffer_bytes.png", buffer_bytes).expect("Failed to write img.png");
//     // std::io::BufWriter::new(std::fs::File::create("img_from_raw_buffer_bytes.png").expect("Failed to create file"))
//         // .write_all(buffer_bytes)
//         // .expect("Failed to write img.png");

//     // let img = PngDecoder::new(buffer_bytes).unwrap();
//     // let mut imagebuffer: RgbImage = ImageBuffer::new(img.dimensions().0, img.dimensions().1);
//     // let _ = img.read_image(imagebuffer.as_mut());
//     // imagebuffer.save("img.png").unwrap();

//     let img = ImageReader::with_format(
//         Cursor::new(buffer_bytes),
//         image::ImageFormat::Png
//         );
//     // let img = ImageReader::new(Cursor::new(buffer_bytes))
//         // .with_guessed_format()
//         // .expect("Failed to guess image format");
//     log!("Image reader format: {:?}", img.format());
//     let img = img.decode().expect("Failed to decode image");
//     // let img = image::load_from_memory(buffer_bytes).expect("Failed to decode image").to_rgb8();
//     log!("Image: {:?}", img);
//     // img.save("img.png").unwrap();
//     // // let img = PngDecoder::new(buffer.to_string()).unwrap();
//     // // log!("Image: {:?}", img);
//     // // let img = img.as_string().unwrap();
// }

#[component]
fn UploadImage() -> impl IntoView {
    // Upload image component and saves into a file "img.png"
    let file_input: NodeRef<Input> = NodeRef::new();

    view! {
        <div>
            <input type="file" accept="image/*" 
                node_ref=file_input
            />
            <button
                on:click=move |_| {
                    // Handle file upload and save to "img.png"
                    // Use the file input to get the image data
                    // Save the image data to a file
                    

                    // Call the denoise function with the image data
                    
                    let file_input_value = file_input.get();
                    spawn_local(async move {
                        PrintFileContent(file_input_value).await;
                    })
                }
                >"Process Image"
                </button>
        </div>
    }
}

#[component]
fn DisplayImage(img_src: String) -> impl IntoView {
    // Display image component
    view! {
        <div>
            <img src=img_src alt="Uploaded Image" />
        </div>
    }
}



#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);

    // Button to upload image
    // Display the image
    // Button to denoise image
    // Display the denoised image

    view! {
        <button
            on:click=move |_| set_count.update(|c| *c += 1)
        >
            "Click me! Count: " {count}
        </button>
        <p>"This is a test of the TGV denoising algorithm."</p>
        <UploadImage />

    }
}

fn main() {
    // println!("Hello, world!");
    // Set up the panic hook to log errors to the console
    console_error_panic_hook::set_once();

    // let img_bytes = std::fs::read("../tgv_denoise_image/astronaut.png").expect("Failed to read image file");
    // println!("Image bytes as string: {:?}", String::from_utf8(img_bytes.clone()).unwrap());

    // let img = ImageReader::new(Cursor::new(img_bytes))
    //     .with_guessed_format()
    //     .expect("Failed to guess image format");
    // let img = img.decode().expect("Failed to decode image");
    // let img = img.to_rgb8();
    // img.save("img.png").unwrap();

    // Main entry point for the application
    leptos::mount::mount_to_body(App)
}
