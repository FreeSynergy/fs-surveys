//! The window: one screen per data type, the standard menus, the words.

use fs_components_fields::Say;
use fs_surveys_app::{Ctx, Words, gui::app_widget};

#[test]
fn the_window_has_a_view_for_each_of_the_four_types() {
    let ctx = Ctx::in_memory("https://example.org/users/owner", "example.org").unwrap();
    let app = app_widget(&ctx).expect("the shell is made");
    app.with_shell(|s| {
        let titles: Vec<&str> = s.menus().iter().map(|m| m.label.as_str()).collect();
        assert_eq!(titles, ["Program", "View", "Help"]);
        let ids: Vec<&str> = s
            .menus()
            .iter()
            .flat_map(|m| m.entries.iter().map(|e| e.id.as_str()))
            .collect();
        for view in [
            "view-Survey",
            "view-Question",
            "view-SurveyResponse",
            "view-Decision",
        ] {
            assert!(ids.contains(&view), "{view} missing in {ids:?}");
        }
    });
}

#[test]
fn words_come_from_the_key_or_the_table() {
    assert_eq!(Words.say("action.save"), "Save");
    assert_eq!(Words.say("question.cmd.set-conditions"), "Set conditions");
    assert_eq!(
        Words.say("survey.anonymity.anonymous-once"),
        "Anonymous once"
    );
}
