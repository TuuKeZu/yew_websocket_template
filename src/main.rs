mod packets;

use anyhow::Error;
use packets::*;

use yew::format::Text;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

struct State {
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

impl Component for State {
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
        let loading_screen = html! {
            <div class="connect-screen">
                <button onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
            </div>
        };

        let game_screen = html! {
            <div class="game-screen">
                /* For frontend testing only - TODO generate runtime */
                <div class="row">
                    <div class="column">
                        <div class="square occupied">
                            <h1>{"O"}</h1>
                        </div>
                    </div>
                    <div class="column">
                        <div class="square occupied">
                            <h1>{"X"}</h1>
                        </div>
                    </div>
                    <div class="column">
                        <div class="square occupied">
                            <h1>{"X"}</h1>
                        </div>
                    </div>
                </div>
                <div class="row">
                    <div class="column">
                        <div class="square empty">
                            <h1>{""}</h1>
                        </div>
                    </div>
                    <div class="column">
                        <div class="square empty">
                            <h1>{""}</h1>
                        </div>
                    </div>
                    <div class="column">
                        <div class="square occupied">
                            <h1>{"X"}</h1>
                        </div>
                    </div>
                </div>
                <div class="row">
                    <div class="column">
                        <div class="square occupied">
                            <h1>{"O"}</h1>
                        </div>
                    </div>
                    <div class="column">
                        <div class="square occupied">
                            <h1>{"X"}</h1>
                        </div>
                    </div>
                    <div class="column">
                        <div class="square occupied">
                            <h1>{"O"}</h1>
                        </div>
                    </div>
                </div>
            </div>
        };

        html! {
            <div class="container">
                {
                    if !self.connected {
                        game_screen
                    }
                    else {
                        loading_screen
                    }
                }
            </div>
        }
    }
}

pub fn to_json(data: PacketType) -> String {
    serde_json::to_string(&data).unwrap()
}

fn main() {
    yew::start_app::<State>();
}
