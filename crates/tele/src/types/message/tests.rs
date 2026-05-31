use std::error::Error as StdError;

use serde_json::{Value, json};

use crate::types::common::{MessageId, ParseMode, UserId};

use super::metadata::KNOWN_MESSAGE_KINDS;
use super::*;

#[test]
fn detects_primary_text_message_kind() -> std::result::Result<(), Box<dyn StdError>> {
    let message: Message = serde_json::from_value(json!({
        "message_id": 1,
        "date": 1700000000,
        "chat": {"id": 1, "type": "private"},
        "text": "hello"
    }))?;

    assert_eq!(message.kind(), MessageKind::Text);
    assert_eq!(message.kinds(), vec![MessageKind::Text]);
    assert!(message.has_kind(MessageKind::Text));
    Ok(())
}

#[test]
fn includes_secondary_caption_kind() -> std::result::Result<(), Box<dyn StdError>> {
    let message: Message = serde_json::from_value(json!({
        "message_id": 2,
        "date": 1700000001,
        "chat": {"id": 1, "type": "private"},
        "photo": [{
            "file_id": "p1",
            "file_unique_id": "u1",
            "width": 16,
            "height": 16
        }],
        "caption": "preview"
    }))?;

    assert_eq!(message.kind(), MessageKind::Photo);
    assert_eq!(
        message.kinds(),
        vec![MessageKind::Photo, MessageKind::Caption]
    );
    assert!(message.has_kind(MessageKind::Caption));
    Ok(())
}

#[test]
fn marks_unmodeled_content_as_unknown() -> std::result::Result<(), Box<dyn StdError>> {
    let message: Message = serde_json::from_value(json!({
        "message_id": 3,
        "date": 1700000002,
        "chat": {"id": 1, "type": "private"},
        "passport_data": {"kind": "mystery"}
    }))?;

    assert_eq!(message.kind(), MessageKind::Unknown);
    assert_eq!(message.kinds(), vec![MessageKind::Unknown]);
    assert!(message.has_kind(MessageKind::Unknown));
    Ok(())
}

#[test]
fn keeps_unknown_alongside_modeled_kind() -> std::result::Result<(), Box<dyn StdError>> {
    let message: Message = serde_json::from_value(json!({
        "message_id": 4,
        "date": 1700000003,
        "chat": {"id": 1, "type": "private"},
        "text": "hello",
        "passport_data": {"kind": "mystery"}
    }))?;

    assert_eq!(message.kind(), MessageKind::Text);
    assert_eq!(
        message.kinds(),
        vec![MessageKind::Text, MessageKind::Unknown]
    );
    assert!(message.has_kind(MessageKind::Unknown));
    Ok(())
}

fn base_message_payload() -> serde_json::Map<String, Value> {
    let mut object = serde_json::Map::new();
    object.insert("message_id".to_owned(), json!(99));
    object.insert("date".to_owned(), json!(1700000999));
    object.insert("chat".to_owned(), json!({"id": 1, "type": "private"}));
    object
}

fn sticker_payload() -> Value {
    json!({
        "file_id": "sticker-1",
        "file_unique_id": "sticker-u-1",
        "type": "regular",
        "width": 64,
        "height": 64,
        "is_animated": false,
        "is_video": false
    })
}

fn gift_payload() -> Value {
    json!({
        "id": "gift-1",
        "sticker": sticker_payload(),
        "star_count": 25
    })
}

fn unique_gift_payload() -> Value {
    json!({
        "gift_id": "gift-1",
        "base_name": "Base Gift",
        "name": "BaseGift-1",
        "number": 1,
        "model": {
            "name": "Model",
            "sticker": sticker_payload(),
            "rarity_per_mille": 500
        },
        "symbol": {
            "name": "Symbol",
            "sticker": sticker_payload(),
            "rarity_per_mille": 250
        },
        "backdrop": {
            "name": "Backdrop",
            "colors": {
                "center_color": 1,
                "edge_color": 2,
                "symbol_color": 3,
                "text_color": 4
            },
            "rarity_per_mille": 125
        }
    })
}

fn message_for_kind(kind: MessageKind) -> std::result::Result<Message, Box<dyn StdError>> {
    let mut object = base_message_payload();
    match kind {
        MessageKind::WriteAccessAllowed => {
            object.insert(
                "write_access_allowed".to_owned(),
                json!({"from_request": true}),
            );
        }
        MessageKind::WebAppData => {
            object.insert(
                "web_app_data".to_owned(),
                json!({"data": "payload", "button_text": "open"}),
            );
        }
        MessageKind::ConnectedWebsite => {
            object.insert("connected_website".to_owned(), json!("example.com"));
        }
        MessageKind::Poll => {
            object.insert(
                "poll".to_owned(),
                json!({
                    "id": "poll-1",
                    "question": "q?",
                    "question_entities": [{"type": "custom_emoji", "offset": 0, "length": 1, "custom_emoji_id": "ce-1"}],
                    "options": [{
                        "persistent_id": "opt-1",
                        "text": "a",
                        "text_entities": [{"type": "bold", "offset": 0, "length": 1}],
                        "voter_count": 1
                    }],
                    "total_voter_count": 1,
                    "is_closed": true,
                    "is_anonymous": false,
                    "type": "regular",
                    "allows_multiple_answers": false,
                    "explanation": "ok",
                    "explanation_entities": [{"type": "bold", "offset": 0, "length": 2}],
                    "open_period": 30,
                    "close_date": 1700001000,
                    "allows_revoting": true
                }),
            );
        }
        MessageKind::PaidMedia => {
            object.insert(
                "paid_media".to_owned(),
                json!({
                    "star_count": 5,
                    "paid_media": [{
                        "type": "preview",
                        "width": 640,
                        "height": 480
                    }]
                }),
            );
        }
        MessageKind::Checklist => {
            object.insert(
                "checklist".to_owned(),
                json!({
                    "title": "ops",
                    "tasks": [{
                        "id": 1,
                        "text": "triage"
                    }],
                    "others_can_add_tasks": true
                }),
            );
        }
        MessageKind::Game => {
            object.insert(
                "game".to_owned(),
                json!({
                    "title": "Demo",
                    "description": "Fun",
                    "photo": [{
                        "file_id": "g-p-1",
                        "file_unique_id": "g-pu-1",
                        "width": 32,
                        "height": 32
                    }]
                }),
            );
        }
        MessageKind::Invoice => {
            object.insert(
                "invoice".to_owned(),
                json!({
                    "title": "Premium",
                    "description": "Subscription",
                    "start_parameter": "start",
                    "currency": "USD",
                    "total_amount": 999
                }),
            );
        }
        MessageKind::SuccessfulPayment => {
            object.insert(
                "successful_payment".to_owned(),
                json!({
                    "currency": "USD",
                    "total_amount": 999,
                    "invoice_payload": "inv-1",
                    "telegram_payment_charge_id": "tg-1",
                    "provider_payment_charge_id": "prov-1"
                }),
            );
        }
        MessageKind::RefundedPayment => {
            object.insert(
                "refunded_payment".to_owned(),
                json!({
                    "currency": "XTR",
                    "total_amount": 100,
                    "invoice_payload": "inv-1",
                    "telegram_payment_charge_id": "tg-1"
                }),
            );
        }
        MessageKind::Gift => {
            object.insert(
                "gift".to_owned(),
                json!({
                    "gift": gift_payload(),
                    "text": "thanks"
                }),
            );
        }
        MessageKind::UniqueGift => {
            object.insert(
                "unique_gift".to_owned(),
                json!({
                    "gift": unique_gift_payload(),
                    "origin": "upgrade"
                }),
            );
        }
        MessageKind::GiftUpgradeSent => {
            object.insert(
                "gift_upgrade_sent".to_owned(),
                json!({
                    "gift": gift_payload(),
                    "is_upgrade_separate": true
                }),
            );
        }
        MessageKind::NewChatMembers => {
            object.insert(
                "new_chat_members".to_owned(),
                json!([{
                    "id": 7,
                    "is_bot": false,
                    "first_name": "newbie"
                }]),
            );
        }
        MessageKind::LeftChatMember => {
            object.insert(
                "left_chat_member".to_owned(),
                json!({
                    "id": 8,
                    "is_bot": false,
                    "first_name": "departed"
                }),
            );
        }
        MessageKind::ChatOwnerLeft => {
            object.insert(
                "chat_owner_left".to_owned(),
                json!({
                    "new_owner": {
                        "id": 9,
                        "is_bot": false,
                        "first_name": "owner-next"
                    }
                }),
            );
        }
        MessageKind::ChatOwnerChanged => {
            object.insert(
                "chat_owner_changed".to_owned(),
                json!({
                    "new_owner": {
                        "id": 10,
                        "is_bot": false,
                        "first_name": "owner-new"
                    }
                }),
            );
        }
        MessageKind::NewChatTitle => {
            object.insert("new_chat_title".to_owned(), json!("ops"));
        }
        MessageKind::NewChatPhoto => {
            object.insert(
                "new_chat_photo".to_owned(),
                json!([{
                    "file_id": "cp-1",
                    "file_unique_id": "cpu-1",
                    "width": 64,
                    "height": 64
                }]),
            );
        }
        MessageKind::DeleteChatPhoto => {
            object.insert("delete_chat_photo".to_owned(), json!(true));
        }
        MessageKind::GroupChatCreated => {
            object.insert("group_chat_created".to_owned(), json!(true));
        }
        MessageKind::SupergroupChatCreated => {
            object.insert("supergroup_chat_created".to_owned(), json!(true));
        }
        MessageKind::ChannelChatCreated => {
            object.insert("channel_chat_created".to_owned(), json!(true));
        }
        MessageKind::PinnedMessage => {
            object.insert(
                "pinned_message".to_owned(),
                json!({
                    "message_id": 500,
                    "date": 0,
                    "chat": {"id": -1001, "type": "supergroup", "title": "mods"}
                }),
            );
        }
        MessageKind::MessageAutoDeleteTimerChanged => {
            object.insert(
                "message_auto_delete_timer_changed".to_owned(),
                json!({"message_auto_delete_time": 60}),
            );
        }
        MessageKind::MigrateToChat => {
            object.insert("migrate_to_chat_id".to_owned(), json!(-1001001001_i64));
        }
        MessageKind::MigrateFromChat => {
            object.insert("migrate_from_chat_id".to_owned(), json!(-1001001002_i64));
        }
        MessageKind::UsersShared => {
            object.insert(
                "users_shared".to_owned(),
                json!({
                    "request_id": 3,
                    "users": [{
                        "user_id": 7,
                        "first_name": "shared"
                    }]
                }),
            );
        }
        MessageKind::ChatShared => {
            object.insert(
                "chat_shared".to_owned(),
                json!({
                    "request_id": 4,
                    "chat_id": -1002,
                    "title": "shared-chat"
                }),
            );
        }
        MessageKind::ProximityAlertTriggered => {
            object.insert(
                "proximity_alert_triggered".to_owned(),
                json!({
                    "traveler": {
                        "id": 11,
                        "is_bot": false,
                        "first_name": "traveler"
                    },
                    "watcher": {
                        "id": 12,
                        "is_bot": false,
                        "first_name": "watcher"
                    },
                    "distance": 42
                }),
            );
        }
        MessageKind::BoostAdded => {
            object.insert("boost_added".to_owned(), json!({"boost_count": 2}));
        }
        MessageKind::ChatBackgroundSet => {
            object.insert(
                "chat_background_set".to_owned(),
                json!({
                    "type": {
                        "type": "fill",
                        "fill": {
                            "type": "solid",
                            "color": 0x112233
                        },
                        "dark_theme_dimming": 50
                    }
                }),
            );
        }
        MessageKind::ChecklistTasksDone => {
            object.insert(
                "checklist_tasks_done".to_owned(),
                json!({
                    "checklist_message": {
                        "message_id": 300,
                        "date": 1700000000,
                        "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                        "checklist": {
                            "title": "ops",
                            "tasks": [{"id": 1, "text": "triage"}]
                        }
                    },
                    "marked_as_done_task_ids": [1]
                }),
            );
        }
        MessageKind::ChecklistTasksAdded => {
            object.insert(
                "checklist_tasks_added".to_owned(),
                json!({
                    "tasks": [{
                        "id": 2,
                        "text": "review"
                    }]
                }),
            );
        }
        MessageKind::DirectMessagePriceChanged => {
            object.insert(
                "direct_message_price_changed".to_owned(),
                json!({
                    "are_direct_messages_enabled": true,
                    "direct_message_star_count": 5
                }),
            );
        }
        MessageKind::ForumTopicCreated => {
            object.insert(
                "forum_topic_created".to_owned(),
                json!({
                    "name": "ops",
                    "icon_color": 7322096
                }),
            );
        }
        MessageKind::ForumTopicEdited => {
            object.insert(
                "forum_topic_edited".to_owned(),
                json!({
                    "name": "ops-renamed"
                }),
            );
        }
        MessageKind::ForumTopicClosed => {
            object.insert("forum_topic_closed".to_owned(), json!({}));
        }
        MessageKind::ForumTopicReopened => {
            object.insert("forum_topic_reopened".to_owned(), json!({}));
        }
        MessageKind::GeneralForumTopicHidden => {
            object.insert("general_forum_topic_hidden".to_owned(), json!({}));
        }
        MessageKind::GeneralForumTopicUnhidden => {
            object.insert("general_forum_topic_unhidden".to_owned(), json!({}));
        }
        MessageKind::GiveawayCreated => {
            object.insert(
                "giveaway_created".to_owned(),
                json!({
                    "prize_star_count": 100
                }),
            );
        }
        MessageKind::Giveaway => {
            object.insert(
                "giveaway".to_owned(),
                json!({
                    "chats": [{
                        "id": -1001,
                        "type": "supergroup",
                        "title": "mods"
                    }],
                    "winners_selection_date": 1700000200,
                    "winner_count": 2,
                    "has_public_winners": true
                }),
            );
        }
        MessageKind::GiveawayWinners => {
            object.insert(
                "giveaway_winners".to_owned(),
                json!({
                    "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                    "giveaway_message_id": 401,
                    "winners_selection_date": 1700000201,
                    "winner_count": 1,
                    "winners": [{
                        "id": 13,
                        "is_bot": false,
                        "first_name": "winner"
                    }]
                }),
            );
        }
        MessageKind::GiveawayCompleted => {
            object.insert(
                "giveaway_completed".to_owned(),
                json!({
                    "winner_count": 3,
                    "is_star_giveaway": true
                }),
            );
        }
        MessageKind::ManagedBotCreated => {
            object.insert(
                "managed_bot_created".to_owned(),
                json!({
                    "bot": {
                        "id": 15,
                        "is_bot": true,
                        "first_name": "managed"
                    }
                }),
            );
        }
        MessageKind::PaidMessagePriceChanged => {
            object.insert(
                "paid_message_price_changed".to_owned(),
                json!({
                    "paid_message_star_count": 7
                }),
            );
        }
        MessageKind::PollOptionAdded => {
            object.insert(
                "poll_option_added".to_owned(),
                json!({
                    "poll_message": {
                        "message_id": 410,
                        "date": 1700000000,
                        "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                        "poll": {
                            "id": "poll-1",
                            "question": "q?",
                            "options": [{"text": "a", "voter_count": 1}],
                            "total_voter_count": 1,
                            "is_closed": false,
                            "is_anonymous": false,
                            "type": "regular",
                            "allows_multiple_answers": false
                        }
                    },
                    "option_persistent_id": "opt-2",
                    "option_text": "new option",
                    "option_text_entities": [{"type": "bold", "offset": 0, "length": 3}]
                }),
            );
        }
        MessageKind::PollOptionDeleted => {
            object.insert(
                "poll_option_deleted".to_owned(),
                json!({
                    "poll_message": {
                        "message_id": 411,
                        "date": 0,
                        "chat": {"id": -1001, "type": "supergroup", "title": "mods"}
                    },
                    "option_persistent_id": "opt-3",
                    "option_text": "old option"
                }),
            );
        }
        MessageKind::SuggestedPostApproved => {
            object.insert(
                "suggested_post_approved".to_owned(),
                json!({
                    "price": {
                        "currency": "XTR",
                        "amount": 50
                    },
                    "send_date": 1700000300
                }),
            );
        }
        MessageKind::SuggestedPostApprovalFailed => {
            object.insert(
                "suggested_post_approval_failed".to_owned(),
                json!({
                    "price": {
                        "currency": "XTR",
                        "amount": 50
                    }
                }),
            );
        }
        MessageKind::SuggestedPostDeclined => {
            object.insert(
                "suggested_post_declined".to_owned(),
                json!({
                    "comment": "no"
                }),
            );
        }
        MessageKind::SuggestedPostPaid => {
            object.insert(
                "suggested_post_paid".to_owned(),
                json!({
                    "currency": "XTR",
                    "star_amount": {
                        "amount": 10
                    }
                }),
            );
        }
        MessageKind::SuggestedPostRefunded => {
            object.insert(
                "suggested_post_refunded".to_owned(),
                json!({
                    "reason": "post_deleted"
                }),
            );
        }
        MessageKind::VideoChatScheduled => {
            object.insert(
                "video_chat_scheduled".to_owned(),
                json!({
                    "start_date": 1700000400
                }),
            );
        }
        MessageKind::VideoChatStarted => {
            object.insert("video_chat_started".to_owned(), json!({}));
        }
        MessageKind::VideoChatEnded => {
            object.insert(
                "video_chat_ended".to_owned(),
                json!({
                    "duration": 120
                }),
            );
        }
        MessageKind::VideoChatParticipantsInvited => {
            object.insert(
                "video_chat_participants_invited".to_owned(),
                json!({
                    "users": [{
                        "id": 14,
                        "is_bot": false,
                        "first_name": "invitee"
                    }]
                }),
            );
        }
        MessageKind::Animation => {
            object.insert(
                "animation".to_owned(),
                json!({
                    "file_id": "anim-1",
                    "file_unique_id": "anim-u-1",
                    "width": 320,
                    "height": 240,
                    "duration": 4
                }),
            );
        }
        MessageKind::Audio => {
            object.insert(
                "audio".to_owned(),
                json!({
                    "file_id": "audio-1",
                    "file_unique_id": "audio-u-1",
                    "duration": 42
                }),
            );
        }
        MessageKind::Contact => {
            object.insert(
                "contact".to_owned(),
                json!({
                    "phone_number": "+123",
                    "first_name": "contact"
                }),
            );
        }
        MessageKind::Dice => {
            object.insert(
                "dice".to_owned(),
                json!({
                    "emoji": "🎲",
                    "value": 6
                }),
            );
        }
        MessageKind::Document => {
            object.insert(
                "document".to_owned(),
                json!({
                    "file_id": "doc-1",
                    "file_unique_id": "doc-u-1"
                }),
            );
        }
        MessageKind::LivePhoto => {
            object.insert(
                "live_photo".to_owned(),
                json!({
                    "photo": [{
                        "file_id": "lp-p-1",
                        "file_unique_id": "lp-pu-1",
                        "width": 16,
                        "height": 16
                    }],
                    "file_id": "live-photo-1",
                    "file_unique_id": "live-photo-u-1",
                    "width": 640,
                    "height": 480,
                    "duration": 3
                }),
            );
        }
        MessageKind::Location => {
            object.insert(
                "location".to_owned(),
                json!({
                    "latitude": 1.25,
                    "longitude": 103.8
                }),
            );
        }
        MessageKind::Photo => {
            object.insert(
                "photo".to_owned(),
                json!([{
                    "file_id": "p-1",
                    "file_unique_id": "u-1",
                    "width": 16,
                    "height": 16
                }]),
            );
        }
        MessageKind::Sticker => {
            object.insert(
                "sticker".to_owned(),
                json!({
                    "file_id": "s-1",
                    "file_unique_id": "su-1",
                    "type": "regular",
                    "width": 128,
                    "height": 128,
                    "is_animated": false,
                    "is_video": false
                }),
            );
        }
        MessageKind::Story => {
            object.insert(
                "story".to_owned(),
                json!({
                    "chat": {"id": -1001, "type": "channel", "title": "stories"},
                    "id": 7
                }),
            );
        }
        MessageKind::Venue => {
            object.insert(
                "venue".to_owned(),
                json!({
                    "location": {
                        "latitude": 1.25,
                        "longitude": 103.8
                    },
                    "title": "HQ",
                    "address": "Somewhere"
                }),
            );
        }
        MessageKind::Video => {
            object.insert(
                "video".to_owned(),
                json!({
                    "file_id": "video-1",
                    "file_unique_id": "video-u-1",
                    "width": 640,
                    "height": 480,
                    "duration": 8
                }),
            );
        }
        MessageKind::VideoNote => {
            object.insert(
                "video_note".to_owned(),
                json!({
                    "file_id": "video-note-1",
                    "file_unique_id": "video-note-u-1",
                    "length": 240,
                    "duration": 8
                }),
            );
        }
        MessageKind::Voice => {
            object.insert(
                "voice".to_owned(),
                json!({
                    "file_id": "voice-1",
                    "file_unique_id": "voice-u-1",
                    "duration": 5
                }),
            );
        }
        MessageKind::Text => {
            object.insert("text".to_owned(), json!("hello"));
        }
        MessageKind::Caption => {
            object.insert("caption".to_owned(), json!("preview"));
        }
        MessageKind::Unknown => {
            object.insert("passport_data".to_owned(), json!({"kind": "mystery"}));
        }
    }

    Ok(serde_json::from_value(Value::Object(object))?)
}

#[test]
fn message_kind_matrix_stays_in_sync() -> std::result::Result<(), Box<dyn StdError>> {
    for &kind in KNOWN_MESSAGE_KINDS {
        let message = message_for_kind(kind)?;
        assert!(
            message.has_kind(kind),
            "missing has_kind mapping for {kind:?}"
        );
        assert!(
            message.kinds().contains(&kind),
            "missing kinds mapping for {kind:?}"
        );
    }
    Ok(())
}

#[test]
fn unknown_kind_matrix_stays_in_sync() -> std::result::Result<(), Box<dyn StdError>> {
    let message = message_for_kind(MessageKind::Unknown)?;
    assert_eq!(message.kind(), MessageKind::Unknown);
    assert!(message.has_kind(MessageKind::Unknown));
    assert_eq!(message.kinds(), vec![MessageKind::Unknown]);
    Ok(())
}

#[test]
fn parses_forward_origin_and_automatic_forward_flag() -> std::result::Result<(), Box<dyn StdError>>
{
    let message: Message = serde_json::from_value(json!({
        "message_id": 42,
        "date": 1700000042,
        "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
        "is_automatic_forward": true,
        "forward_origin": {
            "type": "channel",
            "date": 1700000000,
            "chat": {"id": -1002, "type": "channel", "title": "announcements"},
            "message_id": 777,
            "author_signature": "admin",
            "future": {"kept": true}
        }
    }))?;

    assert!(message.is_automatic_forward());
    let Some(origin) = message.forward_origin() else {
        return Err("missing forward origin".into());
    };
    assert_eq!(origin.kind(), Some("channel"));
    assert_eq!(origin.date(), Some(1_700_000_000));
    assert_eq!(origin.sender_name(), Some("announcements"));
    assert_eq!(origin.message_id(), Some(MessageId(777)));
    assert_eq!(origin.author_signature(), Some("admin"));
    assert_eq!(
        origin
            .as_channel_origin()
            .and_then(|origin| origin.extra.get("future")),
        Some(&json!({"kept": true}))
    );
    assert_eq!(
        origin
            .clone()
            .into_channel_origin()
            .and_then(|origin| origin.extra.get("future").cloned()),
        Some(json!({"kept": true}))
    );

    Ok(())
}

#[test]
fn preserves_unknown_forward_origin() -> std::result::Result<(), Box<dyn StdError>> {
    let origin_payload = json!({
        "type": "future_origin",
        "date": 1700000001,
        "payload": {"kept": true}
    });
    let message: Message = serde_json::from_value(json!({
        "message_id": 43,
        "date": 1700000043,
        "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
        "forward_origin": origin_payload
    }))?;

    let origin = message.forward_origin().ok_or("missing forward origin")?;
    assert_eq!(origin.kind(), Some("future_origin"));
    assert_eq!(origin.as_unknown_value(), Some(&origin_payload));
    assert_eq!(
        origin.clone().into_unknown_value(),
        Some(origin_payload.clone())
    );
    Ok(())
}

#[test]
fn parses_typed_message_entity_and_poll_kinds() -> std::result::Result<(), Box<dyn StdError>> {
    let message: Message = serde_json::from_value(json!({
        "message_id": 43,
        "date": 1700000043,
        "chat": {"id": 1, "type": "private"},
        "text": "/start hello",
        "entities": [
            {"type": "bot_command", "offset": 0, "length": 6},
            {
                "type": "date_time",
                "offset": 7,
                "length": 5,
                "unix_time": 1700000000,
                "date_time_format": "ddd"
            },
            {"type": "mystery_entity", "offset": 13, "length": 5}
        ],
        "poll": {
            "id": "poll-2",
            "question": "q?",
            "question_entities": [{"type": "custom_emoji", "offset": 0, "length": 1, "custom_emoji_id": "ce-2"}],
            "options": [{
                "persistent_id": "opt-2",
                "text": "a",
                "voter_count": 1
            }],
            "total_voter_count": 1,
            "is_closed": true,
            "is_anonymous": false,
            "type": "quiz",
            "allows_multiple_answers": false,
            "correct_option_ids": [0],
            "explanation": "because",
            "explanation_entities": [{"type": "italic", "offset": 0, "length": 7}],
            "close_date": 1700000044,
            "allows_revoting": true
        }
    }))?;

    let entities = message.entities.as_ref().ok_or("missing entities")?;
    assert_eq!(entities[0].kind, MessageEntityKind::BotCommand);
    assert_eq!(entities[1].kind, MessageEntityKind::DateTime);
    assert_eq!(entities[1].unix_time, Some(1_700_000_000));
    assert_eq!(entities[1].date_time_format.as_deref(), Some("ddd"));
    assert_eq!(
        entities[2].kind,
        MessageEntityKind::Unknown("mystery_entity".to_owned())
    );

    let poll = message.poll.as_ref().ok_or("missing poll")?;
    assert_eq!(poll.kind, PollKind::Quiz);
    assert_eq!(poll.correct_option_ids.as_deref(), Some(&[0_u32][..]));
    assert_eq!(poll.options[0].persistent_id.as_deref(), Some("opt-2"));
    assert_eq!(poll.explanation.as_deref(), Some("because"));
    assert_eq!(poll.close_date, Some(1_700_000_044));
    assert_eq!(poll.allows_revoting, Some(true));

    Ok(())
}

#[test]
fn parses_service_message_metadata_and_references() -> std::result::Result<(), Box<dyn StdError>> {
    let message: Message = serde_json::from_value(json!({
        "message_id": 44,
        "date": 1700000044,
        "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
        "sender_chat": {"id": -1002, "type": "channel", "title": "announcements"},
        "sender_boost_count": 3,
        "author_signature": "anonymous admin",
        "sender_tag": "ops",
        "message_thread_id": 77,
        "direct_messages_topic": {
            "topic_id": 7001,
            "user": {"id": 3, "is_bot": false, "first_name": "topic-owner"}
        },
        "is_topic_message": true,
        "via_bot": {"id": 99, "is_bot": true, "first_name": "relay"},
        "has_protected_content": true,
        "is_from_offline": true,
        "is_paid_post": true,
        "media_group_id": "album-1",
        "paid_star_count": 5,
        "quote": {
            "text": "quoted",
            "position": 3,
            "is_manual": true
        },
        "external_reply": {
            "origin": {
                "type": "user",
                "date": 1700000000,
                "sender_user": {"id": 2, "is_bot": false, "first_name": "alice"}
            },
            "message_id": 123,
            "live_photo": {
                "file_id": "external-live-photo-1",
                "file_unique_id": "external-live-photo-u-1",
                "width": 640,
                "height": 480,
                "duration": 3
            }
        },
        "reply_to_story": {
            "chat": {"id": -1002, "type": "channel", "title": "announcements"},
            "id": 77
        },
        "reply_to_checklist_task_id": 9,
        "reply_to_poll_option_id": "opt-1",
        "reply_to_message": {
            "message_id": 10,
            "date": 0,
            "chat": {"id": -1001, "type": "supergroup", "title": "mods"}
        },
        "pinned_message": {
            "message_id": 11,
            "date": 1700000000,
            "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
            "text": "hello"
        },
        "link_preview_options": {"is_disabled": true},
        "reply_markup": {
            "inline_keyboard": [[{"text": "Open", "url": "https://example.com"}]]
        },
        "managed_bot_created": {
            "bot": {"id": 16, "is_bot": true, "first_name": "managed"}
        },
        "poll_option_added": {
            "poll_message": {
                "message_id": 12,
                "date": 1700000000,
                "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
                "poll": {
                    "id": "poll-2",
                    "question": "q?",
                    "options": [{"text": "a", "voter_count": 1}],
                    "total_voter_count": 1,
                    "is_closed": false,
                    "is_anonymous": false,
                    "type": "regular",
                    "allows_multiple_answers": false
                }
            },
            "option_persistent_id": "opt-2",
            "option_text": "added",
            "option_text_entities": [{"type": "italic", "offset": 0, "length": 5}]
        },
        "poll_option_deleted": {
            "poll_message": {
                "message_id": 13,
                "date": 0,
                "chat": {"id": -1001, "type": "supergroup", "title": "mods"}
            },
            "option_persistent_id": "opt-3",
            "option_text": "deleted"
        }
    }))?;

    assert_eq!(message.sender_chat().map(|chat| chat.id), Some(-1002));
    assert_eq!(message.sender_boost_count, Some(3));
    assert_eq!(message.author_signature.as_deref(), Some("anonymous admin"));
    assert_eq!(message.sender_tag.as_deref(), Some("ops"));
    assert_eq!(message.message_thread_id, Some(77));
    assert_eq!(
        message
            .direct_messages_topic
            .as_ref()
            .map(|topic| topic.topic_id),
        Some(7001)
    );
    assert!(message.is_topic_message);
    assert_eq!(
        message.via_bot.as_ref().map(|user| user.id),
        Some(UserId(99))
    );
    assert!(message.has_protected_content);
    assert!(message.is_from_offline);
    assert!(message.is_paid_post);
    assert_eq!(message.media_group_id.as_deref(), Some("album-1"));
    assert_eq!(message.paid_star_count, Some(5));
    assert_eq!(
        message.quote.as_ref().map(|quote| quote.text.as_str()),
        Some("quoted")
    );
    assert_eq!(
        message
            .external_reply
            .as_ref()
            .and_then(|reply| reply.message_id),
        Some(MessageId(123))
    );
    assert_eq!(
        message
            .external_reply
            .as_ref()
            .and_then(|reply| reply.live_photo.as_ref())
            .map(|live_photo| live_photo.file_id.as_str()),
        Some("external-live-photo-1")
    );
    assert_eq!(
        message.reply_to_story.as_ref().map(|story| story.id),
        Some(77)
    );
    assert_eq!(message.reply_to_checklist_task_id, Some(9));
    assert_eq!(message.reply_to_poll_option_id.as_deref(), Some("opt-1"));
    assert_eq!(
        message
            .link_preview_options
            .as_ref()
            .map(|options| options.is_disabled),
        Some(Some(true))
    );
    assert!(message.reply_markup.is_some());
    assert_eq!(
        message
            .managed_bot_created
            .as_ref()
            .map(|event| event.bot.first_name.as_str()),
        Some("managed")
    );
    assert_eq!(
        message
            .poll_option_added
            .as_ref()
            .map(|event| event.option_persistent_id.as_str()),
        Some("opt-2")
    );
    assert_eq!(
        message
            .poll_option_added
            .as_ref()
            .and_then(|event| event.poll_message.as_deref())
            .and_then(MaybeInaccessibleMessage::as_accessible)
            .and_then(|message| message.poll.as_ref())
            .map(|poll| poll.id.as_str()),
        Some("poll-2")
    );
    assert_eq!(
        message
            .poll_option_deleted
            .as_ref()
            .map(|event| event.option_text.as_str()),
        Some("deleted")
    );
    assert!(
        message
            .poll_option_deleted
            .as_ref()
            .and_then(|event| event.poll_message.as_deref())
            .is_some_and(MaybeInaccessibleMessage::is_inaccessible)
    );

    let reply = message.reply_to_message().ok_or("missing reply")?;
    assert!(!reply.is_accessible());
    assert_eq!(reply.message_id(), MessageId(10));

    let pinned = message.pinned_message().ok_or("missing pinned message")?;
    assert!(pinned.is_accessible());
    assert_eq!(
        pinned
            .accessible()
            .and_then(|message| message.text.as_deref()),
        Some("hello")
    );

    Ok(())
}

#[test]
fn parses_chat_background_service_message() -> std::result::Result<(), Box<dyn StdError>> {
    let message: Message = serde_json::from_value(json!({
        "message_id": 46,
        "date": 1700000046,
        "chat": {"id": -1001, "type": "supergroup", "title": "mods"},
        "chat_background_set": {
            "type": {
                "type": "fill",
                "fill": {
                    "type": "gradient",
                    "top_color": 16711680,
                    "bottom_color": 255,
                    "rotation_angle": 90,
                    "future_fill": true
                },
                "dark_theme_dimming": 42,
                "future_type": true
            },
            "future_background": true
        }
    }))?;

    assert_eq!(message.kind(), MessageKind::ChatBackgroundSet);
    let background = message
        .chat_background_set
        .as_ref()
        .ok_or("missing chat background")?;
    assert_eq!(background.background_type.kind(), Some("fill"));
    assert_eq!(background.extra["future_background"], json!(true));
    let fill_background = background
        .background_type
        .as_fill()
        .ok_or("missing fill background")?;
    assert_eq!(fill_background.dark_theme_dimming, 42);
    assert_eq!(fill_background.fill.kind(), Some("gradient"));
    assert_eq!(fill_background.extra["future_type"], json!(true));
    assert_eq!(
        fill_background
            .fill
            .as_gradient()
            .and_then(|fill| fill.extra.get("future_fill")),
        Some(&json!(true))
    );

    Ok(())
}

#[test]
fn chat_background_preserves_unknown_variants() -> std::result::Result<(), Box<dyn StdError>> {
    let background: ChatBackground = serde_json::from_value(json!({
        "type": {
            "type": "future_background",
            "future": true
        }
    }))?;

    assert!(background.background_type.is_unknown());
    assert_eq!(background.background_type.kind(), Some("future_background"));
    assert_eq!(
        background.background_type.as_unknown_value(),
        Some(&json!({
            "type": "future_background",
            "future": true
        }))
    );

    let background: ChatBackground = serde_json::from_value(json!({
        "type": {
            "type": "fill",
            "fill": {
                "type": "future_fill",
                "future": true
            },
            "dark_theme_dimming": 25
        }
    }))?;
    let fill = background
        .background_type
        .as_fill()
        .ok_or("missing fill background")?;
    assert!(fill.fill.is_unknown());
    assert_eq!(fill.fill.kind(), Some("future_fill"));

    Ok(())
}

#[test]
fn parses_paid_media_and_suggested_post_payloads() -> std::result::Result<(), Box<dyn StdError>> {
    let message: Message = serde_json::from_value(json!({
        "message_id": 45,
        "date": 1700000045,
        "chat": {"id": -1003, "type": "channel", "title": "channel"},
        "paid_media": {
            "star_count": 5,
            "paid_media": [{
                "type": "live_photo",
                "live_photo": {
                    "photo": [{
                        "file_id": "lp-1",
                        "file_unique_id": "lpu-1",
                        "width": 32,
                        "height": 32
                    }],
                    "file_id": "lp-video-1",
                    "file_unique_id": "lp-video-u-1",
                    "width": 320,
                    "height": 240,
                    "duration": 3
                },
                "future": {"live": true}
            }, {
                "type": "photo",
                "photo": [{
                    "file_id": "pm-1",
                    "file_unique_id": "pmu-1",
                    "width": 32,
                    "height": 32
                }],
                "future": {"kept": true}
            }]
        },
        "checklist": {
            "title": "ops",
            "tasks": [{
                "id": 1,
                "text": "triage"
            }]
        },
        "invoice": {
            "title": "Premium",
            "description": "Subscription",
            "start_parameter": "start",
            "currency": "USD",
            "total_amount": 999
        },
        "successful_payment": {
            "currency": "USD",
            "total_amount": 999,
            "invoice_payload": "inv-1",
            "telegram_payment_charge_id": "tg-1",
            "provider_payment_charge_id": "prov-1"
        },
        "suggested_post_info": {
            "state": "approved",
            "price": {
                "currency": "XTR",
                "amount": 50
            },
            "send_date": 1700000800
        },
        "suggested_post_refunded": {
            "reason": "payment_refunded"
        }
    }))?;

    assert_eq!(message.kind(), MessageKind::PaidMedia);
    let paid_media = message.paid_media.as_ref().ok_or("missing paid media")?;
    assert_eq!(paid_media.star_count, 5);
    let paid_live_photo = paid_media
        .paid_media
        .first()
        .and_then(PaidMedia::as_live_photo)
        .ok_or("missing paid live photo")?;
    assert_eq!(
        paid_media.paid_media.first().and_then(PaidMedia::kind),
        Some("live_photo")
    );
    assert!(
        paid_media
            .paid_media
            .first()
            .is_some_and(PaidMedia::is_live_photo)
    );
    assert_eq!(paid_live_photo.live_photo.file_id, "lp-video-1");
    assert_eq!(paid_live_photo.extra["future"], json!({"live": true}));
    let paid_media_photo = paid_media
        .paid_media
        .get(1)
        .and_then(PaidMedia::as_photo)
        .ok_or("missing paid media photo")?;
    assert_eq!(
        paid_media.paid_media.get(1).and_then(PaidMedia::kind),
        Some("photo")
    );
    assert!(
        paid_media
            .paid_media
            .get(1)
            .is_some_and(PaidMedia::is_photo)
    );
    assert_eq!(paid_media_photo.photo.len(), 1);
    assert_eq!(paid_media_photo.extra["future"], json!({"kept": true}));
    assert_eq!(
        paid_media
            .paid_media
            .get(1)
            .cloned()
            .and_then(PaidMedia::into_photo)
            .and_then(|value| value.extra.get("future").cloned()),
        Some(json!({"kept": true}))
    );
    assert_eq!(
        paid_media
            .paid_media
            .first()
            .cloned()
            .and_then(PaidMedia::into_live_photo)
            .map(|value| value.live_photo.file_unique_id.clone()),
        Some("lp-video-u-1".to_owned())
    );

    let checklist = message.checklist.as_ref().ok_or("missing checklist")?;
    assert_eq!(checklist.tasks.len(), 1);

    let invoice = message.invoice.as_ref().ok_or("missing invoice")?;
    assert_eq!(invoice.total_amount, 999);

    let payment = message
        .successful_payment
        .as_ref()
        .ok_or("missing successful payment")?;
    assert_eq!(payment.invoice_payload, "inv-1");

    let suggested = message
        .suggested_post_info
        .as_ref()
        .ok_or("missing suggested post info")?;
    assert_eq!(suggested.state, SuggestedPostState::Approved);
    assert_eq!(
        message
            .suggested_post_refunded
            .as_ref()
            .map(|value| &value.reason),
        Some(&SuggestedPostRefundReason::PaymentRefunded)
    );

    Ok(())
}

#[test]
fn preserves_unknown_paid_media_payload() -> std::result::Result<(), Box<dyn StdError>> {
    let input = json!({
        "type": "future_paid_media",
        "payload": {"kept": true}
    });
    let message: Message = serde_json::from_value(json!({
        "message_id": 46,
        "date": 1700000046,
        "chat": {"id": -1003, "type": "channel", "title": "channel"},
        "paid_media": {
            "star_count": 1,
            "paid_media": [input]
        }
    }))?;

    let paid_media = message.paid_media.as_ref().ok_or("missing paid media")?;
    let media = paid_media
        .paid_media
        .first()
        .ok_or("missing paid media item")?;
    assert_eq!(media.kind(), Some("future_paid_media"));
    assert_eq!(media.as_unknown_value(), Some(&input));
    assert_eq!(media.clone().into_unknown_value(), Some(input.clone()));
    Ok(())
}

#[test]
fn input_media_round_trips_with_boxed_variants() -> std::result::Result<(), Box<dyn StdError>> {
    let media = InputMedia::from(InputMediaPhoto {
        media: "attach://photo".to_owned(),
        caption: Some("preview".to_owned()),
        parse_mode: Some(ParseMode::Html),
        caption_entities: None,
        has_spoiler: None,
    });

    let value = serde_json::to_value(&media)?;
    assert_eq!(value.get("type"), Some(&json!("photo")));
    let parsed: InputMedia = serde_json::from_value(value)?;
    let InputMedia::Photo(photo) = parsed else {
        return Err("expected photo input media".into());
    };
    assert_eq!(photo.media, "attach://photo");
    assert_eq!(photo.caption.as_deref(), Some("preview"));

    let live_photo_media = InputMedia::from(InputMediaLivePhoto::new(
        "live-photo-file-id",
        "cover-photo-file-id",
    ));
    let live_photo_value = serde_json::to_value(&live_photo_media)?;
    assert_eq!(live_photo_value.get("type"), Some(&json!("live_photo")));
    let parsed_live_photo: InputMedia = serde_json::from_value(live_photo_value)?;
    let InputMedia::LivePhoto(live_photo) = parsed_live_photo else {
        return Err("expected live photo input media".into());
    };
    assert_eq!(live_photo.media, "live-photo-file-id");
    assert_eq!(live_photo.photo, "cover-photo-file-id");

    Ok(())
}

#[test]
fn edit_message_result_helpers_cover_both_variants() -> std::result::Result<(), Box<dyn StdError>> {
    let message = message_for_kind(MessageKind::Text)?;
    let result = EditMessageResult::from(message.clone());
    assert_eq!(
        result.message().map(|message| message.message_id),
        Some(message.message_id)
    );
    assert_eq!(result.success(), None);
    assert_eq!(
        result.into_message().map(|message| message.message_id),
        Some(message.message_id)
    );

    let success = EditMessageResult::Success(true);
    assert!(success.message().is_none());
    assert_eq!(success.success(), Some(true));

    Ok(())
}
