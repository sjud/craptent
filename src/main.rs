use bytes::Bytes;
use csv::StringRecord;
use gloo::file::ObjectUrl;
use serde::{Deserialize, Deserializer, Serialize};
use dioxus::{prelude::*, core::IntoDynNode};
mod types;
use types::*;
fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    use_shared_state_provider(cx, || serde_json::Map::from_iter(vec![(String::new(),serde_json::Value::Object(serde_json::Map::default()))].into_iter()));
    use_shared_state_provider(cx, || ApiKeys::default());
    let keys = format!("{:?}",use_shared_state::<ApiKeys>(cx).unwrap().read().clone());
    let files_uploaded: &UseRef<Vec<String>> = use_ref(cx, Vec::new);
    let json = use_state(cx, || "{}".to_string());
    let records = use_ref(cx, ||None);
    use_shared_state_provider::<AppState>(cx, || AppState::default());
    let app_state = use_shared_state::<AppState>(cx).unwrap();
    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "Craptent" }
            h3 { "Multimedia AI Content Pipeline Tool" }
            a {href:"https://github.com/sjud/craptent", "Source"}
            p { "Unlock Seamless Integration with Craptent: Your Ultimate AI Content Pipeline and management tool. Effortlessly merge outputs from sources such as ChatGpt, Dall-E, and Elevenlabs into structured data to your frontend with our reliable webhook system." }
            ApiKey {model:GenModel::OpenAI}
            ApiKey {model:GenModel::ElevenLabs}
           p {keys}
           h5 {"Upload CSV"}
           input {
            // tell the input to pick a file
            r#type:"file",
            // list the accepted extensions
            accept: ".csv",
            // pick multiple files
            multiple: true,
            onchange: |evt| {
                to_owned![files_uploaded,records];
            async move {
                if let Some(file_engine) = &evt.files {
                    let files = file_engine.files();
                    for file_name in &files {
                        // Make sure to use async/await when doing heavy I/O operations,
                        // to not freeze the interface in the meantime
                        if let Some(file) = file_engine.read_file_to_string(file_name).await{
                            let mut rdr = csv::Reader::from_reader(file.as_bytes());
                            let r = rdr.into_records();
                            let mut res = vec![];
                            for rec in r {
                                res.push(rec.unwrap())
                            }
                            *records.write() = Some(res.into_iter());
                            files_uploaded.write().push(file);
                        }
                    }
                }
            }
            }
            }
            h5 {"Build Json Structure"}
            BuildJsonStructure{}
            p {format!("{:?}",app_state.read().current_record)}
            button{
                onclick: move |_| {
                    if let Some(records) = records.write().as_mut() {
                        if let Some(record) = records.next() {
                            app_state.write().update_current_record(record.clone());
                        }
                    }
                },
                "next"
            }
           ChatGpt{}
           DallE{}
           ElevenLabs{}
        }
    ))
}

fn recursive_obj_search(
    map: &mut dyn Iterator<Item=(&String, &serde_json::Value)>, 
    list: &mut Vec<LazyNodes>, 
    path: Vec<String>,
) {
    while let Some((field, value)) = map.next() {
        if value.is_object() {
            let mut new_path = path.clone();
            new_path.push(field.clone());
            // Assuming you have an rsx! macro and BuildJsonObject struct defined somewhere
            let new_path_clone = new_path.clone();
            list.push(rsx!{
                BuildJsonObject{
                    path: new_path_clone
                }
            });
            recursive_obj_search(&mut value.as_object().unwrap().into_iter(), list,new_path);
        }
    }
}
fn BuildJsonStructure(cx:Scope) -> Element {
    let map = use_shared_state::<serde_json::Map<String,serde_json::Value>>(cx).unwrap();
    let mut add_list = vec![];
    let mut path = vec![];
    recursive_obj_search(&mut map.read().iter(),&mut add_list,path);
    cx.render(rsx!{
        p{
            "{serde_json::to_string(&*map.read().get(\"\").unwrap()).unwrap()}"
        }
        add_list.into_iter()
    })
}

#[derive(Props,PartialEq)]
pub struct BuildJsonObjectProps{
    path:Vec<String>,
}

fn BuildJsonObject(cx:Scope<BuildJsonObjectProps>) -> Element {
    use serde_json::Value;
    let map = use_shared_state::<serde_json::Map<String,Value>>(cx).unwrap();
    let field_name = use_state(cx, || "".to_string());
    let value = use_state(cx,||"bool".to_string());
    let with = use_state(cx,||"".to_string());
    let obj_name = cx.props.path.join(".");
    cx.render(rsx!{
        p{ "Add Field to {obj_name}" }
        input{
            oninput: move |evt| field_name.set(evt.value.clone()),
        }
        span{"Value:"}
        select{
            onchange: move |evt| value.set(evt.value.clone()),
            option {
                value:"bool",
                "Bool"
            },
            option {
                value:"number",
                "Number"
            },
            option {
                value: "string",
                "String"
            },
            option {
                value:"array-bool",
                "ArrayBool"
            },
            option {
                value:"array-number",
                "ArrayNumber"
            },
            option {
                value:"array-string",
                "ArrayString"
            },
            option {
                value: "object",
                "object"
            },
        },
        span{"With:"}
        input{
            oninput: move |evt| with.set(evt.value.clone()),
        }
        button{
            onclick:move |_| {
                let field_name = field_name.current().as_ref().clone();
                let value = match value.current().as_ref().clone().as_str() {
                    "bool" => serde_json::Value::Bool(with.current().as_ref().clone().parse::<bool>().unwrap_or_default()),
                    "number" => serde_json::Value::Number(serde_json::Number::from_f64(with.current().as_ref().clone().parse::<f64>().unwrap_or_default()).unwrap()),
                    "string" => serde_json::Value::String(with.current().as_ref().clone().parse::<String>().unwrap_or_default()),
                    "array-bool" => serde_json::Value::Array(with.current().as_ref().clone().split(",").map(|s|Value::Bool(s.parse::<bool>().unwrap_or_default())).collect::<Vec<Value>>()),
                    "array-number" => serde_json::Value::Array(with.current().as_ref().clone().split(",").map(|s|
                        Value::Number(serde_json::Number::from_f64(s.parse::<f64>().unwrap_or_default()).unwrap()
                        )).collect::<Vec<serde_json::Value>>()),
                    "array-string" => serde_json::Value::Array(with.current().as_ref().clone().split(",").map(|s|Value::String(s.to_string())).collect::<Vec<Value>>()),
                    "object" => serde_json::Value::Object(serde_json::Map::<String,serde_json::Value>::default()),
                    _ => panic!("unexpected value for value"),
                };
                let mut map = map.write();
                let mut map = map.get_mut(&cx.props.path[0]).unwrap();
                let mut i = 1; 
                    while cx.props.path.get(i).is_some() {
                        map = map.get_mut(&cx.props.path[i]).unwrap();
                        i+=1;
                    }
                    if let serde_json::Value::Object(map) = map {
                        map.insert(field_name,value);
                    } else {
                        panic!("expected path to resolve to a map");
                    }
            },
            style:"width:3em;height:2em;",
            "Add"
        }
        }
    )
}

fn ApiKey(cx:Scope<ApiKeyProps>) -> Element {
    let keys = use_shared_state::<ApiKeys>(cx).unwrap();
    let key = use_state(cx, || "".to_string());
    let title = match cx.props.model {
        GenModel::OpenAI => "OpenAI Key",
        GenModel::ElevenLabs => "ElevenLabs Key",
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
                    GenModel::OpenAI => current.open_ai = key.get().clone(),
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


async fn fetch_chat_gpt(
    model_response:UseSharedState<CompletionResponse>,
    key:String,
    model:String,
    frequency_penalty:f32,
    max_tokens:u32,
    batch_size:u8,
    presence_penalty:f32,
    stop_sequence:Vec<String>,
    temperature:f32,
    top_p:f32,
    system:String,
    prompt:String,
    ) {
    let resp = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization",format!("Bearer {}",key))
        .header("Content-Type","application/json")
        .body(format!("{{
            \"model\":\"{}\",
            \"frequency_penalty\":{},
            \"max_tokens\":{},
            \"n\":{},
            \"presence_penalty\":{},
            \"stop\":\"[{}]\",
            \"temperature\":{},
            \"top_p\":{},
            \"messages\":[
                {{\"role\":\"system\",\"content\":\"{}\"}},
                {{\"role\":\"user\",\"content\":\"{}\"}}
                ]
        }}",
        model,
        frequency_penalty,
        max_tokens,
        batch_size,
        presence_penalty,
        stop_sequence.join(","),
        temperature,
        top_p,
        system,
        prompt 
    ))
    .send()
    .await
    .unwrap()
    .json::<CompletionResponse>()
    .await
    .unwrap();
    *model_response.write() = resp;
}


fn ChatGpt(cx:Scope) -> Element {
    use_shared_state_provider(cx, || CompletionResponse::default());
    let app_state = use_shared_state::<AppState>(cx).unwrap();
    let model_resp = use_shared_state::<CompletionResponse>(cx).unwrap();
    let keys = use_shared_state::<ApiKeys>(cx).unwrap();
    let batch_size = use_state(cx, || 1);
    let temperature = use_state(cx, || 1.);
    let max_tokens = use_state(cx, || 256);
    let stop_sequence: &UseState<Vec<String>> = use_state(cx, || vec![]);
    let top_p = use_state(cx, || 1.);
    let frequency_penalty = use_state(cx, || 0.);
    let presence_penalty = use_state(cx, || 0.);
    let model = use_state(cx, ||  "gpt-3.5-turbo".to_string());
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
            h3{"ChatGPT"}

            div {
                p {
                    "System"
                }
                textarea {
                    value: "{app_state.read().chat_gpt_system_raw}",
                    oninput: move |evt| app_state.write().update_field(AppStateFieldUpdate::ChatGPTSystem(evt.value.clone())),
                },
                p {
                    "{app_state.read().chat_gpt_system_edited}"
                }
            },
            div {
                 p {
                    "Prompt"
                }
                textarea {
                    value: "{app_state.read().chat_gpt_prompt_raw}",
                    oninput: move |evt| app_state.write().update_field(AppStateFieldUpdate::ChatGPTPrompt(evt.value.clone())),
                },
                p {
                    "{app_state.read().chat_gpt_prompt_edited}"
                }
            },
            
           div {
            p {
                "Model"
            }
            select {
                onchange: move |evt| model.set(evt.value.clone()),

                option {
                    value:"gpt-3.5-turbo",
                    "gpt-3.5-turbo"
                },
                option {
                    value:"gpt-4",
                    "gpt-4"
                },
                option {
                    value: "gpt-3.5-turbo-16k",
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
               "Max Tokens"
           }
           input {
               value: "{max_tokens}",
               oninput: move |evt| max_tokens.set(evt.value.clone().parse::<u32>().unwrap_or_default().min(4092)),
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
               oninput: move |evt| presence_penalty.set(evt.value.clone().parse::<f32>().unwrap_or_default().max(-2.).min(2.)),
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
       div {
        button{
            style: "width:6em;height:2em;",
            onclick: move |_| {
                    fetch_chat_gpt(
                        model_resp.clone(),
                        (*keys).read().open_ai.clone(),
                        model.current().as_ref().clone(),
                        frequency_penalty.current().as_ref().clone(),
                        max_tokens.current().as_ref().clone(),
                        batch_size.current().as_ref().clone(),
                        presence_penalty.current().as_ref().clone(),
                        stop_sequence.current().as_ref().clone(),
                        temperature.current().as_ref().clone(),
                        top_p.current().as_ref().clone(),
                        app_state.read().chat_gpt_system_edited.clone(),
                        app_state.read().chat_gpt_prompt_edited.clone(),
                    )
            },
            "Submit"
        }
       }
       div {
        if (*model_resp.read()) != CompletionResponse::default() {
            rsx!(MessageChoices{
                choices:(*model_resp.read()).message_choices.clone()
            })
        }

       }
        }
    )
}


fn MessageChoices(cx:Scope<MessageChoicesProps>) -> Element {
    use serde_json::Value;
    let map = use_shared_state::<serde_json::Map<String,Value>>(cx).unwrap();
    let path = use_state(cx, || "".to_string());
    cx.render(rsx!(
        for choice in &cx.props.choices {
            p{
                "{choice.message.content.clone()}"
            }
            input{
                value:"{path}",
                oninput: move |evt| path.set(evt.value.clone())
            }
            button{
                onclick: move |_| {
                    let mut map_c = map.read().clone();
                    let mut temp = map_c.get_mut("").unwrap().as_object_mut().unwrap();
                    for p in path.current().split(".") {
                        let new_temp = temp.get_mut(p).unwrap();
                        if new_temp.is_string() {
                            log::error!("is string");
                            *new_temp = Value::String(choice.message.content.clone());
                            break;
                        }
                        temp = new_temp.as_object_mut().unwrap();
                       
                    }
                    *map.write() = map_c;
                },
                style:"width:10em;height:2em;",
                "add to field"
            }
        }
    ))
}
async fn fetch_dall_e(
    model_response:UseSharedState<DallEResponse>,
    key:String,
    size:String,
    batch_size:u8,
    prompt:String,
    ) {
    let resp = reqwest::Client::new()
        .post("https://api.openai.com/v1/images/generations")
        .header("Authorization",format!("Bearer {}",key))
        .header("Content-Type","application/json")
        .body(format!("{{
            \"n\":{},
            \"size\":\"{}\",
            \"prompt\":\"{}\"
        }}",
        batch_size,
        size,
        prompt 
    ))
    .send()
    .await
    .unwrap()
    .json::<DallEResponse>()
    .await
    .unwrap();
    *model_response.write() = resp;
}


fn DallE(cx:Scope) -> Element {
    use_shared_state_provider(cx, || DallEResponse::default());
    let app_state = use_shared_state::<AppState>(cx).unwrap();
    let model_resp = use_shared_state::<DallEResponse>(cx).unwrap();
    let prompt = use_state(cx, || "".to_string());
    let batch_size = use_state(cx, || 1);
    let size = use_state(cx, || "256x256".to_string());
    let keys = use_shared_state::<ApiKeys>(cx).unwrap();

    cx.render(
        rsx!{
            h3{"Dall-E"}

            div {
                p {
                   "Prompt"
               }
               textarea {
                   value: "{prompt}",
                   oninput: move |evt| app_state.write().update_field(AppStateFieldUpdate::DallE(evt.value.clone())),
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
        div {
            p {
                "Size"
            }
            select {
                onchange: move |evt| size.set(evt.value.clone()),
                option {
                    value: "256x256",
                    "256x256"
                },
                option {
                    value:"512x512",
                    "512x512"
                },
                option {
                    value:"1024x1024",
                    "1024x1024"
                    },  
                },
           }
           div {
            button{
                style: "width:6em;height:2em;",
                onclick: move |_| {
                        fetch_dall_e(
                            model_resp.clone(),
                            (*keys).read().open_ai.clone(),
                            size.current().as_ref().clone(),
                            batch_size.current().as_ref().clone(),
                            prompt.current().as_ref().clone() 
                        )
                },
                "Submit"
            }
           }
           div {
                (*model_resp.read()).data.iter().map(|img|
                    {
                        let url = &img.url;
                        rsx!(
                            img { key: "{url}", src: "{url}" }
                        )
                    }
                )
            }
        }
    )
}

pub async fn text_to_audio(
    model_response:UseSharedState<Vec<ObjectUrl>>,
    key:String,
    voice_id:String,
    text:String,
    voice_settings:VoiceSettings,
) {
    let bytes = reqwest::Client::new()
        .post(&format!("https://api.elevenlabs.io/v1/text-to-speech/{}",voice_id))
        .header("xi-api-key",key)  
        .header("Content-Type","application/json")
        .body(format!("{{
            \"text\":\"{}\",
            \"model_id\":\"eleven_multilingual_v1\",
            \"voice_settings\":{}
        }}",
        text,
        serde_json::to_string(&voice_settings).unwrap(),
    ))    
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let blob = gloo::file::Blob::new_with_options(&*bytes,Some("mpeg/audio"));
    let object_url = ObjectUrl::from(blob);
    *model_response.write() = vec![object_url];
}

pub fn ElevenLabs(cx:Scope) -> Element {
    use_shared_state_provider::<Vec<ObjectUrl>>(cx, || vec![]);
    let app_state = use_shared_state::<AppState>(cx).unwrap();
    let model_resp = use_shared_state::<Vec<ObjectUrl>>(cx).unwrap();
    let keys = use_shared_state::<ApiKeys>(cx).unwrap();
    let similarity_boost = use_state(cx, || 0.70);
    let stability = use_state(cx, || 0.70);
    let style = use_state(cx, || 0.20);
    let use_speaker_boost = use_state(cx, || false);
    let voice_id = use_state(cx, || "".to_string());
    let future_voices = use_future(cx, (&keys.read().eleven_labs), 
    |key| async move {
        if key.is_empty() {
            None
        } else {
            Some(reqwest::Client::new()
                .get("https://api.elevenlabs.io/v1/voices")
                .header("xi-api-key",key)      
                .send()
                .await
                .unwrap()
                .json::<VoicesResponse>()
                .await
                .unwrap())
        }
    });

    cx.render(rsx!{
        h3{"ElevenLabs"}
        match future_voices.value() {
            Some(resp) => {
                if let Some(resp) = resp {
                    rsx!{
                        select{
                            onchange: move |evt| voice_id.set(evt.value.clone()),
                            option{
                                value:"",
                                "EMPTY",
                            } 
                            resp.voices.iter().map(|voice|
                                rsx!{
                                    option{
                                        value:voice.voice_id.as_str(),
                                        voice.name.as_str()
                                    } 
                                }
                            )
                        }
                        div {
                            p {
                               "Stability"
                           }
                           input {
                               value: "{stability}",
                               oninput: move |evt| stability.set(evt.value.clone().parse::<f64>().unwrap_or_default().max(0.).min(1.)),
                           },
                        }
                        div {
                            p {
                               "Similarity"
                           }
                           input {
                               value: "{similarity_boost}",
                               oninput: move |evt| similarity_boost.set(evt.value.clone().parse::<f64>().unwrap_or_default().max(0.).min(1.)),
                           },
                        }
                        div {
                            p {
                               "Style"
                           }
                           input {
                               value: "{style}",
                               oninput: move |evt| style.set(evt.value.clone().parse::<f64>().unwrap_or_default().max(0.).min(1.)),
                           },
                        }
                        div {
                            p {
                               "Speaker Boost"
                           }
                           input {
                               r#type:"checkbox",
                               onchange: move |_| use_speaker_boost.set(!use_speaker_boost.current().as_ref()),
                           },
                        }
                        div {
                            p {
                               "Text"
                            }
                            textarea {
                                value: "{app_state.read().eleven_labs_raw}",
                                oninput: move |evt| app_state.write().update_field(AppStateFieldUpdate::ElevenLabs(evt.value.clone())),
                            },
                            p {
                                "{app_state.read().eleven_labs_edited}"
                            }
                        }
                        div {
                            button{
                                style: "width:6em;height:2em;",
                                onclick: move |_| {
                                        text_to_audio(
                                            model_resp.clone(),
                                            (*keys).read().eleven_labs.clone(),
                                            voice_id.current().as_ref().clone(),
                                            app_state.read().dall_e_edited.clone(),
                                            VoiceSettings { 
                                                similarity_boost: similarity_boost.current().as_ref().clone(), 
                                                stability: stability.current().as_ref().clone(),
                                                 style: style.current().as_ref().clone(), 
                                                 use_speaker_boost: use_speaker_boost.current().as_ref().clone(), 
                                                }
                                        )
                                },
                                "Submit"
                            }
                       }
                       (*model_resp.read()).iter().map(|obj|
                        {
                            let obj = obj.clone();
                            let url = obj.parse::<String>().unwrap();
                            rsx!(
                                audio { key: "{url}", src: "{url}", controls: true }
                            )
                        }
                    )
                    }
                } else {
                    rsx!{
                        p {
                            "Set your ElevenLabs Key to get your voices."
                        }
                    }
                }
            },
            None => {
                rsx!{
                    p {
                        "Set your ElevenLabs Key to get your voices."
                    }
                }
            }
        }
    })
}