use std::borrow::Borrow;

use dioxus::prelude::*;

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    use_shared_state_provider(cx, || ApiKeys::default());
    let keys = 
        format!("{:?}",use_shared_state::<ApiKeys>(cx).unwrap().read().clone());
    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "Craptent" }
            h3 { "AI Content Pipeline Tool" }
            p { "Unlock Seamless Integration with Craptent: Your Ultimate AI Content Pipeline and management tool. Effortlessly merge outputs from sources such as ChatGpt, Midjourney, and Elevenlabs into structured data to your frontend with our reliable webhook system." }
            ApiKey {model:GenModel::ChatGPT}
            ApiKey {model:GenModel::ElevenLabs}
            ApiKey {model:GenModel::Midjourney}
           p {keys}
           ChatGpt{}
        }

    ))
}
#[derive(Default,Clone,Debug,PartialEq)]
pub struct ApiKeys{
    chat_gpt:String,
    midjourney:String,
    eleven_labs:String,
}
#[derive(Props,PartialEq)]
struct ApiKeyProps{
    model:GenModel,
}
#[derive(Clone,Debug,Copy,PartialEq)]
pub enum GenModel{
    ChatGPT,
    Midjourney,
    ElevenLabs,
}

fn ApiKey(cx:Scope<ApiKeyProps>) -> Element {
    let keys = use_shared_state::<ApiKeys>(cx).unwrap();
    let key = use_state(cx, || "".to_string());
    let title = match cx.props.model {
        GenModel::ChatGPT => "ChatGPT Key",
        GenModel::Midjourney => "Midjourney Key",
        GenModel::ElevenLabs => "Elevenlabs Key",
    };
    cx.render(
        rsx!{
            div {
            div {
                "{title}"
            }
            input {
                value: "{key}",
                oninput: move |evt| key.set(evt.value.clone()),
            }
            button {
                style: "width:3em;height:2em;",
                onclick : move |_| {
                    let mut current = keys.read().clone();
                    match cx.props.model {
                    GenModel::ChatGPT => current.chat_gpt = key.get().clone(),
                    GenModel::Midjourney => current.midjourney = key.get().clone(),
                    GenModel::ElevenLabs => current.eleven_labs = key.get().clone(),
                    }
                    *keys.write() = current;
                },
                "Set"
            }
            }

        }
    )
}


fn ChatGpt(cx:Scope) -> Element {
    let system = use_state(cx, || "".to_string());
    let prompt = use_state(cx, || "".to_string());
    let batch_size = use_state(cx, || 1);
    let temperature = use_state(cx, || 1.);
    let max_length = use_state(cx, || 256);
    let stop_sequence: &UseState<Vec<String>> = use_state(cx, || vec![]);
    let top_p = use_state(cx, || 1.);
    let frequency_penalty = use_state(cx, || 0.);
    let presence_penalty = use_state(cx, || 0.);
    let model = use_state(cx, || "3.5".to_string());
    let sequence = use_state(cx, || "".to_string());

    let mut stop_sequence_rendered = vec![];
    for seq in stop_sequence.current()
    .iter() {
        let seq = seq.clone();
        stop_sequence_rendered.push(
            rsx!{
                button {
                    onclick : move |evt| {
                        let current = stop_sequence.current().iter().filter_map(|s| if s != &seq{
                            Some(s.clone())
                        } else {
                            None
                        }).collect();
                        stop_sequence.set(current);
                    },
                    "X {seq}"
                }
            }
        )
    }
    cx.render(
        rsx!{
            div {
                p {
                    "System"
                }
                input {
                    value: "{system}",
                    oninput: move |evt| system.set(evt.value.clone()),
                },
            },
            div {
                 p {
                    "Prompt"
                }
                input {
                    value: "{prompt}",
                    oninput: move |evt| prompt.set(evt.value.clone()),
                },
            }
          
           div {
            p {
                "Model"
            }
            select {
                onchange: move |evt| model.set(evt.value.clone()),

                option {
                    value:"4",
                    "gpt-4"
                },
                option {
                    value:"3.5",
                    "gpt-3.5-turbo"
                },
                option {
                    value:"3.5-16k",
                    "gpt-3.5-turbo-16k"
                },
            },
           }
        div {
            p {
               "Temperature"
           }
           input {
               value: "{temperature}",
               oninput: move |evt| temperature.set(evt.value.clone().parse::<f32>().unwrap_or_default().max(0.).min(2.)),
           },
        }
        div {
            p {
               "Max Length"
           }
           input {
               value: "{max_length}",
               oninput: move |evt| max_length.set(evt.value.clone().parse::<u32>().unwrap_or_default().min(4092)),
           },
        }
        div {
            p {
                "Stop Sequence"
            }
            form {
                onsubmit : move |evt| {
                        let mut current : Vec<String> = stop_sequence.current().clone().iter().map(|s|s.clone()).collect();
                        current.push(sequence.current().as_ref().clone());
                        stop_sequence.set(current);
                        sequence.set("".to_string()) 
                    },
                input { 
                    value:"{sequence}",
                    oninput: move |evt| sequence.set(evt.value.clone()),
                },
                input { r#type: "submit", value:"Add" },
            }
            {stop_sequence_rendered.into_iter()}
        }
        div {
            p {
               "Top P"
           }
           input {
               value: "{top_p}",
               oninput: move |evt| top_p.set(evt.value.clone().parse::<f32>().unwrap_or_default().max(0.).min(1.)),
           },
        }
        div {
            p {
               "Frequency Penalty"
           }
           input {
               value: "{frequency_penalty}",
               oninput: move |evt| frequency_penalty.set(evt.value.clone().parse::<f32>().unwrap_or_default().max(0.).min(2.)),
           },
        }
        div {
            p {
               "Presence Penalty"
           }
           input {
               value: "{presence_penalty}",
               oninput: move |evt| presence_penalty.set(evt.value.clone().parse::<f32>().unwrap_or_default().max(0.).min(2.)),
           },
        }
        div {
            p {
               "Batch Size"
           }
           input {
               value: "{batch_size}",
               oninput: move |evt| batch_size.set(evt.value.clone().parse::<u8>().unwrap_or_default()),
           },
       }
        }
    )
}