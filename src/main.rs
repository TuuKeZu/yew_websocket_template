mod packets;

use anyhow::Error;
use packets::*;

use yew::format::Text;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

struct Model {
    ws: Option<WebSocketTask>,
    link: ComponentLink<Self>,

    connected: bool,
    chat: Vec<String>,
    input: String,
}
enum Msg {
    Connect,
    Disconnected,
    Connected,
    Received(Result<String, Error>),
    OnInput(String),
    SendMessage(),
    Error(String),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            ws: None,
            link,
            connected: false,
            chat: Vec::new(),
            input: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Connect => {
                ConsoleService::log("Connecting");

                let cbout = self.link.callback(Msg::Received);
                let cbnot = self.link.callback(|input| match input {
                    WebSocketStatus::Closed => Msg::Disconnected,
                    WebSocketStatus::Error => {
                        Msg::Error("Failed to connect to servers".to_string())
                    }
                    _ => Msg::Connected,
                });
                if self.ws.is_none() {
                    let task = WebSocketService::connect_text(
                        "ws://127.0.0.1:8090/c05554ae-b4ee-4976-ac05-97aaf3c98a24",
                        cbout,
                        cbnot,
                    );
                    self.ws = Some(task.unwrap());
                }
                true
            }
            Msg::Disconnected => {
                self.ws = None;
                self.connected = false;
                true
            }
            Msg::Connected => {
                self.connected = true;
                true
            }
            Msg::Received(Ok(s)) => {
                let json: Result<PacketType, serde_json::Error> = serde_json::from_str(&s);

                if let Ok(packet) = json {
                    match packet {
                        PacketType::Message(content) => {
                            self.chat.push(content);
                        }
                        PacketType::Error(_, content) => {
                            self.chat.push(content);
                        }
                    }
                }

                true
            }
            Msg::Received(Err(s)) => {
                ConsoleService::error(&format!("Received invalid data from the server! {}", s));
                false
            }
            Msg::Error(e) => {
                self.chat.push(e);
                false
            }
            Msg::SendMessage() => match self.ws {
                Some(ref mut task) => {
                    task.send::<Text>(Text::into(Ok(to_json(PacketType::Message(
                        self.input.clone(),
                    )))));
                    self.input = String::new();
                    true
                }
                None => false,
            },
            Msg::OnInput(content) => {
                self.input = content;
                true
            }
        }
    }

    fn change(&mut self, _prop: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
                <div class="connect-screen" style={format!("display: {}", if !self.connected {"flex"} else {"none"})}>
                    <button onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
                </div>
                <div class="chat-screen" /*  style={format!("display: {}", if self.connected {"flex"} else {"none"})}*/>
                    <div class="message-list">
                        {
                            for self.chat.iter().map(|content| {

                                html! {
                                    <div class="message">
                                        <h1>{content}</h1>
                                    </div>
                                }
                            })
                        }
                    </div>
                    <div class="input-area">
                        <input value={self.input.clone()} type="text" placeholder="Send a message" oninput=self.link.callback(|e: InputData| Msg::OnInput(e.value))/><br/>
                        <button onclick=self.link.callback(|_| Msg::SendMessage())>{ "Send" }</button>
                    </div>
                </div>
            </div>
        }
    }
}

pub fn to_json(data: PacketType) -> String {
    serde_json::to_string(&data).unwrap()
}

fn main() {
    yew::start_app::<Model>();
}
