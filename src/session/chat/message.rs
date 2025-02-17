use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdgrand::enums::{MessageContent, MessageSender, Update};
use tdgrand::types::Message as TelegramMessage;

use crate::session::Chat;

#[derive(Clone, Debug, PartialEq, glib::GBoxed)]
#[gboxed(type_name = "BoxedMessageContent")]
pub struct BoxedMessageContent(pub MessageContent);

#[derive(Clone, Debug, glib::GBoxed)]
#[gboxed(type_name = "BoxedMessageSender")]
pub struct BoxedMessageSender(MessageSender);

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct Message {
        pub id: Cell<i64>,
        pub sender: OnceCell<MessageSender>,
        pub outgoing: Cell<bool>,
        pub date: Cell<i32>,
        pub content: RefCell<Option<BoxedMessageContent>>,
        pub chat: OnceCell<Chat>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Message {
        const NAME: &'static str = "ChatMessage";
        type Type = super::Message;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Message {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_int64(
                        "id",
                        "Id",
                        "The id of this message",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "sender",
                        "Sender",
                        "The sender of this message",
                        BoxedMessageSender::static_type(),
                        glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boolean(
                        "outgoing",
                        "Outgoing",
                        "Wheter this message is outgoing or not",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_int(
                        "date",
                        "Date",
                        "The point in time when this message was sent",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "content",
                        "Content",
                        "The content of this message",
                        BoxedMessageContent::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpec::new_object(
                        "chat",
                        "Chat",
                        "The chat relative to this message",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "id" => {
                    let id = value.get().unwrap();
                    self.id.set(id);
                }
                "sender" => {
                    let sender = value.get::<BoxedMessageSender>().unwrap();
                    self.sender.set(sender.0).unwrap();
                }
                "outgoing" => {
                    let outgoing = value.get().unwrap();
                    self.outgoing.set(outgoing);
                }
                "date" => {
                    let date = value.get().unwrap();
                    self.date.set(date);
                }
                "content" => {
                    let content = value.get().unwrap();
                    self.content.replace(Some(content));
                }
                "chat" => {
                    let chat = value.get().unwrap();
                    self.chat.set(chat).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "outgoing" => self.outgoing.get().to_value(),
                "date" => self.date.get().to_value(),
                "content" => self.content.borrow().as_ref().unwrap().to_value(),
                "chat" => self.chat.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Message(ObjectSubclass<imp::Message>);
}

impl Message {
    pub fn new(message: TelegramMessage, chat: &Chat) -> Self {
        let content = BoxedMessageContent(message.content);
        let sender = BoxedMessageSender(message.sender);
        glib::Object::new(&[
            ("id", &message.id),
            ("sender", &sender),
            ("outgoing", &message.is_outgoing),
            ("date", &message.date),
            ("content", &content),
            ("chat", chat),
        ])
        .expect("Failed to create Message")
    }

    pub fn handle_update(&self, update: Update) {
        match update {
            Update::MessageContent(update) => {
                let new_content = BoxedMessageContent(update.new_content);
                self.set_content(new_content);
            }
            _ => {}
        }
    }

    pub fn id(&self) -> i64 {
        self.property("id").unwrap().get().unwrap()
    }

    pub fn sender(&self) -> &MessageSender {
        let self_ = imp::Message::from_instance(self);
        self_.sender.get().unwrap()
    }

    pub fn outgoing(&self) -> bool {
        self.property("outgoing").unwrap().get().unwrap()
    }

    pub fn date(&self) -> i32 {
        self.property("date").unwrap().get().unwrap()
    }

    pub fn content(&self) -> BoxedMessageContent {
        self.property("content").unwrap().get().unwrap()
    }

    fn set_content(&self, content: BoxedMessageContent) {
        if self.content() != content {
            self.set_property("content", &content).unwrap();
        }
    }

    pub fn chat(&self) -> Chat {
        self.property("chat").unwrap().get().unwrap()
    }
}
