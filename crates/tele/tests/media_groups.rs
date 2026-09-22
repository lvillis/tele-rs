use tele::Error;
use tele::types::{
    InputMediaAudio, InputMediaDocument, InputMediaGroupItem, InputMediaLivePhoto, InputMediaPhoto,
    InputMediaVideo, SendMediaGroupRequest,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn media_groups_reject_mixed_audio_and_document_albums() -> TestResult {
    let invalid_pairs: [[InputMediaGroupItem; 2]; 4] = [
        [
            InputMediaAudio::new("audio").into(),
            InputMediaPhoto::new("photo").into(),
        ],
        [
            InputMediaDocument::new("doc").into(),
            InputMediaVideo::new("video").into(),
        ],
        [
            InputMediaAudio::new("audio").into(),
            InputMediaDocument::new("doc").into(),
        ],
        [
            InputMediaDocument::new("doc").into(),
            InputMediaLivePhoto::new("live", "photo").into(),
        ],
    ];
    for pair in invalid_pairs {
        for media in [pair.to_vec(), pair.into_iter().rev().collect()] {
            assert!(matches!(SendMediaGroupRequest::new(1_i64, media.clone()),
                Err(Error::InvalidRequest { reason }) if reason.contains("same type")));
            let mut request = SendMediaGroupRequest::new(
                1_i64,
                vec![
                    InputMediaPhoto::new("one").into(),
                    InputMediaPhoto::new("two").into(),
                ],
            )?;
            request.media = media;
            assert!(
                matches!(request.validate(), Err(Error::InvalidRequest { reason })
                if reason.contains("same type"))
            );
        }
    }
    Ok(())
}

#[test]
fn media_groups_allow_visual_mixes_and_homogeneous_audio_or_documents() -> TestResult {
    for media in [
        vec![
            InputMediaPhoto::new("photo").into(),
            InputMediaVideo::new("video").into(),
            InputMediaLivePhoto::new("live", "cover").into(),
        ],
        vec![
            InputMediaAudio::new("one").into(),
            InputMediaAudio::new("two").into(),
        ],
        vec![
            InputMediaDocument::new("one").into(),
            InputMediaDocument::new("two").into(),
        ],
    ] {
        SendMediaGroupRequest::new(1_i64, media)?.validate()?;
    }
    Ok(())
}
