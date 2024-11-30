use leptos::*;

#[derive(Clone, Debug)]
pub enum MessageSeverity {
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug)]
pub struct Message {
    id: u32,
    text: String,
    severity: MessageSeverity,
}

#[derive(Clone)]
pub struct MessageContext {
    messages: RwSignal<Vec<Message>>,
    next_id: RwSignal<u32>,
}

impl MessageContext {
    pub fn new() -> Self {
        Self {
            // messages: create_rw_signal(Vec::new()),
            messages: create_rw_signal(vec![
                Message {
                    id: 0,
                    text: "Welcome to Proof of Tests!".into(),
                    severity: MessageSeverity::Info,
                },
                Message {
                    id: 1,
                    text: "Some features may be under development".into(),
                    severity: MessageSeverity::Warn,
                },
                Message {
                    id: 2,
                    text: "Unable to connect to server".into(),
                    severity: MessageSeverity::Error,
                },
            ]),
            next_id: create_rw_signal(3),
        }
    }

    pub fn add(&self, text: impl Into<String>, severity: MessageSeverity) {
        let id = self.next_id.get();
        self.next_id.set(id + 1);

        self.messages.update(|messages| {
            messages.push(Message {
                id,
                text: text.into(),
                severity,
            });
        });
    }

    pub fn remove(&self, id: u32) {
        self.messages.update(|messages| {
            messages.retain(|msg| msg.id != id);
        });
    }
}

#[component]
pub fn Messages() -> impl IntoView {
    let message_ctx = expect_context::<MessageContext>();

    view! {
        <div class="fixed top-4 left-1/2 -translate-x-1/2 z-50 space-y-2 max-w-2xl w-full px-4">
            {move || message_ctx.messages.get().into_iter().map(|message| {
                let message_ctx = message_ctx.clone();
                let id = message.id;

                let bg_color = match message.severity {
                    MessageSeverity::Info => "bg-blue-100 text-blue-800",
                    MessageSeverity::Warn => "bg-yellow-100 text-yellow-800",
                    MessageSeverity::Error => "bg-red-100 text-red-800",
                };

                view! {
                    <div
                        class=format!("p-4 rounded-lg shadow-md flex justify-between items-start {}", bg_color)
                        role="alert"
                    >
                        <span>{message.text}</span>
                        <button
                            class="ml-4 hover:opacity-70"
                            on:click=move |_| message_ctx.remove(id)
                        >
                            "×"
                        </button>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
