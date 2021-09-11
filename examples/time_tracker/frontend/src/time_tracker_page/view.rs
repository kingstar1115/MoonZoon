use zoon::*;
use crate::{theme::Theme, app};
use std::{sync::Arc, convert::TryFrom};

// @TODO try rewrite some clone()s to references (applies to all pages)
// @TODO refactor some parts to shared views in app.rs (applies to all pages)
// @TODO const DELETE_ENTITY_BUTTON_RADIUS in app.rs? + replace 40 / 2 with it
// @TODO dark/light theme (applies to all pages)
// @TODO fixed header? (applies to all pages)
// @TODO CorId SessionId custom Debug impl?

pub fn page() -> impl Element {
    Column::new()
        .item(title())
        .item(content())
}

fn title() -> impl Element {
    El::new()
        .s(Width::fill().max(600))
        .s(Padding::new().y(35))
        .child(
            El::with_tag(Tag::H1)
                .s(Align::center())
                .s(Font::new().size(30).weight(NamedWeight::SemiBold))
                .child("Time Tracker")
        )
}

fn content() -> impl Element {
    Column::new()
        .s(Spacing::new(35))
        .s(Padding::new().x(10).bottom(10))
        .item(clients())
}

// -- clients --

fn clients() -> impl Element {
    Column::new()
        .s(Spacing::new(35))
        .s(Align::new().center_x())
        .items_signal_vec(super::clients().signal_vec_cloned().map(client))
}

fn client(client: Arc<super::Client>) -> impl Element {
    Column::new()
        .s(Background::new().color(Theme::Background1))
        .s(RoundedCorners::all(10))
        .s(Padding::all(15))
        .s(Spacing::new(20))
        .item(client_name(client.clone()))
        .item(projects(client))
}

fn client_name(client: Arc<super::Client>) -> impl Element {
    El::new()
        .s(Width::fill())
        .s(Font::new().color(Theme::Font1).size(20))
        .s(Background::new().color(Theme::Transparent))
        .s(Padding::all(8))
        .child(&client.name)
}

// -- projects --

fn projects(client: Arc<super::Client>) -> impl Element {
    Column::new()
        .s(Spacing::new(20))
        .items(client.projects.iter().map(|p| {
            project(p.clone())
        }))
}

fn project(project: Arc<super::Project>) -> impl Element {
    Column::new()
        .s(Background::new().color(Theme::Background0))
        .s(RoundedCorners::all(10))
        .s(Spacing::new(20))
        .s(Padding::all(10))
        .item(project_name_and_start_stop_button(project.clone()))
        .item(time_entries(project))
}

fn project_name_and_start_stop_button(project: Arc<super::Project>) -> impl Element {
    Row::new()
        .item(project_name(project.clone()))
        .item(start_stop_button(project))
}

fn project_name(project: Arc<super::Project>) -> impl Element {
    El::new()
        .s(Width::fill())
        .s(Font::new().color(Theme::Font0).size(18))
        .s(Background::new().color(Theme::Transparent))
        .s(Padding::all(8))
        .child(&project.name)
}

fn start_stop_button(project: Arc<super::Project>) -> impl Element {
    let mutable_has_active_entry = Mutable::new(false);
    let has_active_entry = mutable_has_active_entry.read_only();
    let has_active_entry_updater = Task::start_droppable(
        project
            .time_entries
            .signal_vec_cloned()
            .filter_signal_cloned(|time_entry| {
                time_entry.stopped.signal().map(|stopped| stopped.is_none())
            })
            .len()
            .map(|active_entries_count| active_entries_count > 0)
            .for_each(move |has_active_entry| {
                mutable_has_active_entry.set_neq(has_active_entry);
                // @TODO for_each_sync?
                async {}
            })
    );

    let (hovered, hovered_signal) = Mutable::new_and_signal(false);
    let background_color = map_ref! {
        let hovered = hovered_signal,
        let has_active_entry = has_active_entry.signal() =>
        match (has_active_entry, hovered) {
            (true, false) => Theme::Background4,
            (true, true) => Theme::Background4Highlighted,
            (false, false) => Theme::Background3,
            (false, true) => Theme::Background3Highlighted,
        }
    };

    Button::new()
        .s(Background::new().color_signal(background_color))
        .s(Font::new()
            .color_signal(has_active_entry.signal().map_bool(|| Theme::Font4, || Theme::Font3))
        )
        .s(RoundedCorners::all_max())
        .s(Padding::new().x(20).y(10))
        .after_remove(move |_| drop(has_active_entry_updater))
        .on_hovered_change(move |is_hovered| hovered.set_neq(is_hovered))
        .on_press(move || super::toggle_tracker(&project))
        .label_signal(has_active_entry.signal().map_bool(|| "Stop", || "Start"))
}

// -- time_entries --

fn time_entries(project: Arc<super::Project>) -> impl Element {
    Column::new()
        .s(Spacing::new(20))
        .items_signal_vec(project.time_entries.signal_vec_cloned().map(move |t| {
            time_entry(project.clone(), t.clone())
        }))
}

fn time_entry(project: Arc<super::Project>, time_entry: Arc<super::TimeEntry>) -> impl Element {
    let is_active_mutable = Mutable::new(false);
    let is_active = is_active_mutable.read_only();
    let is_active_updater = Task::start_droppable(time_entry
        .stopped
        .signal()
        .for_each(move |stopped| {
            is_active_mutable.set_neq(stopped.is_none());
            async { }
        })
    );
    Column::new()
        .s(Background::new()
            .color_signal(is_active.signal().map_bool(|| Theme::Background4, || Theme::Background1))
        )
        .s(RoundedCorners::all(10).top_right(40 / 2))
        .s(Padding::new().bottom(15))
        .after_remove(move |_| drop(is_active_updater))
        .item(time_entry_name_and_delete_button(project, time_entry.clone(), is_active.clone()))
        .item_signal(time_entry_times(time_entry, is_active))
}

fn time_entry_name_and_delete_button(
    project: Arc<super::Project>, 
    time_entry: Arc<super::TimeEntry>,
    is_active: ReadOnlyMutable<bool>,
) -> impl Element {
    let id = time_entry.id;
    Row::new()
        .item(time_entry_name(time_entry.clone(), is_active.clone()))
        .item(delete_entity_button(move || super::delete_time_entry(&project, id), is_active))
}

fn time_entry_name(time_entry: Arc<super::TimeEntry>, is_active: ReadOnlyMutable<bool>) -> impl Element {
    let debounced_rename = Mutable::new(None);
    El::new()
        .s(Padding::all(10))
        .child(
            TextInput::new()
                .s(Width::fill())
                .s(Font::new()
                    .color_signal(is_active.signal().map_bool(|| Theme::Font4, || Theme::Font1))
                )
                .s(Background::new().color(Theme::Transparent))
                .s(Borders::new().bottom_signal(
                    is_active.signal().map_bool(
                        || Border::new().color(Theme::Border4), 
                        || Border::new().color(Theme::Border1)
                    )
                ))
                .s(Padding::all(5))
                .focus(not(time_entry.is_old))
                .label_hidden("time_entry name")
                .text_signal(time_entry.name.signal_cloned())
                .on_change(move |text| {
                    time_entry.name.set_neq(text);
                    debounced_rename.set(Some(Timer::once(app::DEBOUNCE_MS, move || {
                        super::rename_time_entry(time_entry.id, &time_entry.name.lock_ref())
                    })))
                })
        )
}

fn delete_entity_button(on_press: impl FnOnce() + Clone + 'static, is_active: ReadOnlyMutable<bool>) -> impl Element {
    let (hovered, hovered_signal) = Mutable::new_and_signal(false);
    let background_color = map_ref! {
        let hovered = hovered_signal,
        let is_active = is_active.signal() =>
        match (is_active, hovered) {
            (true, false) => Theme::Background1,
            (true, true) => Theme::Background1Highlighted,
            (false, false) => Theme::Background3,
            (false, true) => Theme::Background3Highlighted,
        }
    };
    Button::new()
        .s(Width::new(40))
        .s(Height::new(40))
        .s(Align::new().top().right())
        .s(Background::new().color_signal(background_color))
        .s(Font::new()
            .color_signal(is_active.signal().map_bool(|| Theme::Font1, || Theme::Font3))
            .weight(NamedWeight::Bold)
        )
        .s(RoundedCorners::all_max())
        .on_hovered_change(move |is_hovered| hovered.set_neq(is_hovered))
        .on_press(on_press)
        .label(app::icon_delete_forever())
}

fn time_entry_times(time_entry: Arc<super::TimeEntry>, is_active: ReadOnlyMutable<bool>) -> impl Signal<Item = RawElement> {
    super::show_wide_time_entry().map(move |show_wide| {
        let items = element_vec![
            time_entry_started(time_entry.clone(), is_active.clone()),
            time_entry_duration(time_entry.clone(), is_active.clone()),
            time_entry_stopped(time_entry.clone(), is_active.clone()),
        ];
        if show_wide {
            time_entry_times_wide(items, is_active.clone()).into_raw_element()
        } else {
            time_entry_times_narrow(items, is_active.clone()).into_raw_element()
        }
    })
}

fn time_entry_times_narrow(items: Vec<RawElement>, is_active: ReadOnlyMutable<bool>) -> impl Element {
    Column::new()
        .s(
            Font::new()
                .color_signal(is_active.signal().map_bool(|| Theme::Font4, || Theme::Font1))
        )
        .items(items)
}

fn time_entry_times_wide(items: Vec<RawElement>, is_active: ReadOnlyMutable<bool>) -> impl Element {
    Row::new()
        .s(
            Font::new()
                .color_signal(is_active.signal().map_bool(|| Theme::Font4, || Theme::Font1))
        )
        .s(Padding::new().x(10))
        .s(Spacing::new(20))
        .items(items)
}

fn time_entry_date(
    year: impl Signal<Item = i32> + Unpin + 'static, 
    month: impl Signal<Item = u32> + Unpin + 'static, 
    day: impl Signal<Item = u32> + Unpin + 'static,
    is_active: ReadOnlyMutable<bool>,
    read_only_when_active: bool,
) -> impl Element {
    Row::new()
        .s(Align::new().center_x())
        .s(Spacing::new(2))
        .item(
            date_time_part_input(
                year, 
                4, 
                false,
                is_active.clone(),
                read_only_when_active,
            )
        )
        .item("-")
        .item(
            date_time_part_input(
                month.map(|month| i32::try_from(month).unwrap_throw()), 
                2, 
                false,
                is_active.clone(),
                read_only_when_active,
            )
        )
        .item("-")
        .item(
            date_time_part_input(
                day.map(|day| i32::try_from(day).unwrap_throw()), 
                2, 
                false,
                is_active,
                read_only_when_active,
            )
        )
}

fn time_entry_time(
    hour: impl Signal<Item = u32> + Unpin + 'static, 
    minute: impl Signal<Item = u32> + Unpin + 'static, 
    second: impl Signal<Item = u32> + Unpin + 'static,
    is_active: ReadOnlyMutable<bool>,
    read_only_when_active: bool,
) -> impl Element {
    Row::new()
        .s(Align::new().center_x())
        .s(Spacing::new(2))
        .item(
            date_time_part_input(
                hour.map(|hour| i32::try_from(hour).unwrap_throw()), 
                2, 
                false, 
                is_active.clone(),
                read_only_when_active,
            ),
        )
        .item(":")
        .item(
            date_time_part_input(
                minute.map(|minute| i32::try_from(minute).unwrap_throw()), 
                2, 
                false,
                is_active.clone(),
                read_only_when_active,
            )
        )
        .item(":")
        .item(
            date_time_part_input(
                second.map(|second| i32::try_from(second).unwrap_throw()), 
                2, 
                false,
                is_active,
                read_only_when_active,
            )
        )
}

fn time_entry_started(time_entry: Arc<super::TimeEntry>, is_active: ReadOnlyMutable<bool>) -> impl Element {
    Row::new()
        .s(Padding::all(5))
        .s(Spacing::new(15))
        .item(time_entry_started_date(time_entry.clone(), is_active.clone()))
        .item(time_entry_started_time(time_entry.clone(), is_active))
}

fn time_entry_started_date(time_entry: Arc<super::TimeEntry>, is_active: ReadOnlyMutable<bool>) -> impl Element {
    let year = time_entry.started.signal().map(|date| date.year());
    let month = time_entry.started.signal().map(|date| date.month());
    let day = time_entry.started.signal().map(|date| date.day());
    time_entry_date(year, month, day, is_active, false)
}

fn time_entry_started_time(time_entry: Arc<super::TimeEntry>, is_active: ReadOnlyMutable<bool>) -> impl Element {
    let hour = time_entry.started.signal().map(|time| time.hour());
    let minute = time_entry.started.signal().map(|time| time.minute());
    let second = time_entry.started.signal().map(|time| time.second());
    time_entry_time(hour, minute, second, is_active, false)
}

fn time_entry_duration(time_entry: Arc<super::TimeEntry>, is_active: ReadOnlyMutable<bool>) -> impl Element {
    let mutable_duration = Mutable::new((0, 0, 0));
    let duration = mutable_duration.read_only();
    let duration_signal = map_ref! {
        let current = super::current_time().signal(),
        let started = time_entry.started.signal(),
        let stopped = time_entry.stopped.signal() =>
        if let Some(stopped) = stopped {
            **stopped - **started
        } else {
            *current - **started
        }
    }.dedupe();
    let duration_updater = Task::start_droppable(
        duration_signal.for_each(move |duration| {
            let num_seconds = duration.num_seconds();
            let seconds = i32::try_from(num_seconds % 60).unwrap_throw();
            let minutes = i32::try_from((num_seconds / 60) % 60).unwrap_throw();
            let hours = i32::try_from((num_seconds / 60) / 60).unwrap_throw();
            mutable_duration.set((hours, minutes, seconds));
            async {} 
        })
    );
    let hours = duration.signal().map(|(hours, _, _)| hours);
    let minutes = duration.signal().map(|(_, minutes, _)| minutes);
    let seconds = duration.signal().map(|(_, _, seconds)| seconds);

    Row::new()
        .s(Align::new().center_x())
        .s(Padding::all(5))
        .s(Spacing::new(10))
        .after_remove(move |_| drop(duration_updater))
        .item(
            Row::new()
                .s(Spacing::new(2))
                .item(date_time_part_input(hours, None, true, is_active.clone(), true))
                .item("h"))
        .item(
            Row::new()
                .s(Spacing::new(2))
                .item(date_time_part_input(minutes, 2, true, is_active.clone(), true))
                .item("m"))
        .item(
            Row::new()
                .s(Spacing::new(2))
                .item(date_time_part_input(seconds, 2, true, is_active, true))
                .item("s"))
}

fn time_entry_stopped(time_entry: Arc<super::TimeEntry>, is_active: ReadOnlyMutable<bool>) -> impl Element {
    Row::new()
        .s(Padding::all(5))
        .s(Spacing::new(15))
        .item(time_entry_stopped_date(time_entry.clone(), is_active.clone()))
        .item(time_entry_stopped_time(time_entry.clone(), is_active))
}

fn time_entry_stopped_date(time_entry: Arc<super::TimeEntry>, is_active: ReadOnlyMutable<bool>) -> impl Element {
    let year = map_ref! {
        let current_date = super::current_time().signal(),
        let stopped_date = time_entry.stopped.signal() =>
        if let Some(stopped_date) = stopped_date {
            stopped_date.year()
        } else {
            current_date.year()
        }
    }.dedupe();
    let month = map_ref! {
        let current_date = super::current_time().signal(),
        let stopped_date = time_entry.stopped.signal() =>
        if let Some(stopped_date) = stopped_date {
            stopped_date.month()
        } else {
            current_date.month()
        }
    }.dedupe();
    let day = map_ref! {
        let current_date = super::current_time().signal(),
        let stopped_date = time_entry.stopped.signal() =>
        if let Some(stopped_date) = stopped_date {
            stopped_date.day()
        } else {
            current_date.day()
        }
    }.dedupe();
    time_entry_date(year, month, day, is_active, true)
}

fn time_entry_stopped_time(time_entry: Arc<super::TimeEntry>, is_active: ReadOnlyMutable<bool>) -> impl Element {
    let hour = map_ref! {
        let current_time = super::current_time().signal(),
        let stopped_time = time_entry.stopped.signal() =>
        if let Some(stopped_time) = stopped_time {
            stopped_time.hour()
        } else {
            current_time.hour()
        }
    }.dedupe();
    let minute = map_ref! {
        let current_time = super::current_time().signal(),
        let stopped_time = time_entry.stopped.signal() =>
        if let Some(stopped_time) = stopped_time {
            stopped_time.minute()
        } else {
            current_time.minute()
        }
    }.dedupe();
    let second = map_ref! {
        let current_time = super::current_time().signal(),
        let stopped_time = time_entry.stopped.signal() =>
        if let Some(stopped_time) = stopped_time {
            stopped_time.second()
        } else {
            current_time.second()
        }
    }.dedupe();
    time_entry_time(hour, minute, second, is_active, true)
}

fn date_time_part_input(
    number: impl Signal<Item = i32> + Unpin + 'static, 
    max_chars: impl Into<Option<u32>>, 
    bold: bool,
    is_active: ReadOnlyMutable<bool>,
    read_only_when_active: bool, 
) -> impl Element {
    let max_chars = max_chars.into();
    TextInput::new()
        .s(Width::zeros(max_chars.unwrap_or(4)))
        .s(
            Font::new()
                .color_signal(is_active.signal().map_bool(|| Theme::Font4, || Theme::Font1))
                .center()
                .weight(if bold { NamedWeight::Bold } else { NamedWeight::Regular } )
        )
        .s(Background::new().color(Theme::Transparent))
        .s(Borders::new().bottom(
            Border::new().color(Theme::Border1)
        ))
        .s(Borders::new().bottom_signal(
            is_active.signal().map_bool(
                move || Border::new().color(if read_only_when_active { Theme::Transparent } else { Theme::Border4 }), 
                || Border::new().color(Theme::Border1),
            )
        ))
        .label_hidden("time entry started date")
        .text_signal(number.map(move |number| {
            if max_chars == Some(2) {
                format!("{:02}", number)
            } else {
                number.to_string()
            }
        }))
        .input_type(InputType::text().max_chars(max_chars))
        .read_only_signal(is_active.signal().map_bool(move || read_only_when_active, || false))
}





// blocks!{

//     #[el]
//     fn page() -> Column {
//         column![
//             el![
//                 region::h1(),
//                 "Time Tracker",
//             ],
//             client_panels();
//         ]
//     }

//     // ------ Client ------

//     #[el]
//     fn client_panels() -> Column {
//         let clients = super::clients().map(|clients| {
//             clients.map(|clients| clients.iter_vars().rev().map(client_panel))
//         });
//         column![
//             spacing(30),
//             clients,
//         ]
//     }

//     #[el]
//     fn client_panel(client: Var<super::Client>) -> Column {
//         column![
//             el![client.map(|client| client.name.clone())],
//             project_panels(client),
//         ]
//     }

//     // ------ Project ------

//     #[el]
//     fn project_panels(client: Var<super::Client>) -> Column {
//         let projects = client.map(|client| {
//             client.projects.iter_vars().rev().map(project_panel)
//         });
//         column![
//             spacing(20),
//             projects,
//         ]
//     }

//     #[el]
//     fn project_panel(project: Var<super::Project>) -> Column {
//         column![
//             row![
//                 el![project.map(|project| project.name.clone())],
//                 start_stop_button(project),
//             ],
//             time_entry_panels(project),
//         ]
//     }

//     #[el]
//     fn start_stop_button(project: Var<super::Project>) -> Button {
//         if let Some(time_entry) = project.map(|project| project.active_time_entry) {
//             button![
//                 background::color(color::yellow()),
//                 button::on_press(|| super::set_time_entry_stopped(time_entry, Local::now())),
//                 "Stop",
//             ]
//         } else {
//             button![
//                 background::color(color::green()),
//                 button::on_press(|| super::add_time_entry(project)),
//                 "Start",
//             ]
//         }
//     }

//     // ------ TimeEntry ------

//     #[el]
//     fn time_entry_panels(project: Var<super::Project>) -> Column {
//         let time_entries = project.map(|project| {
//             project.time_entries.iter_vars().rev().map(time_entry_panel)
//         });
//         column![
//             spacing(20),
//             time_entries,
//         ]
//     }

//     #[el]
//     fn time_entry_panel(time_entry: Var<super::TimeEntry>) -> Column {
//         let show_duration_row = app::viewport_width().inner() < DURATION_BREAKPOINT;
//         let active = time_entry.map(|time_entry| time_entry.stopped.is_none());

//         if active {
//             el_var(|| Timer::new(1_000, || {
//                 notify(RecomputeDuration);
//                 notify(RecomputeStopped);
//             }))
//         }

//         column![
//             row![
//                 time_entry_name(time_entry),
//                 button![
//                     button::on_press(|| super::remove_time_entry(time_entry)),
//                     "D",
//                 ],
//             ],
//             show_duration_row.then(|| {
//                 row![
//                     duration_input(time_entry)
//                 ]
//             }),
//             row![
//                 started_inputs(time_entry),
//                 show_duration_row.not().then(|| {
//                     column![
//                         duration_input(time_entry)
//                     ]
//                 }),
//                 stopped_inputs(time_entry),
//             ]
//         ]
//     }

//     #[el]
//     fn time_entry_name(time_entry: Var<super::TimeEntry>) -> TextInput {
//         let name = el_var(|| {
//             time_entry.map(|time_entry| time_entry.name.clone())
//         });
//         text_input![
//             text_input::on_change(|new_name| name.set(new_name)),
//             on_blur(|| name.use_ref(|name| {
//                 super::rename_time_entry(time_entry, name);
//             })),
//             name.inner(),
//         ]
//     }

//     #[el]
//     fn duration_input(time_entry: Var<super::TimeEntry>) -> TextInput {
//         let (active, started, stopped) = time_entry.map(|time_entry| (
//             time_entry.stopped.is_none(),
//             time_entry.started,
//             time_entry.stopped.unwrap_or_else(Local::now)
//         ));
//         let recompute = listen_ref(|RecomputeDuration| ()).is_some();
//         let duration = el_var_reset(recompute, || (stopped - started).hhmmss());
//         // 3:40:20
//         text_input![
//             active.not().then(|| text_input::on_change(|new_duration| duration.set(new_duration))),
//             active.not().then(|| on_blur(|| {
//                 let new_duration = (|| {
//                     let duration = duration.inner();
//                     let negative = duration.chars().next()? == '-';
//                     if negative {
//                         duration.remove(0);
//                     }
//                     let mut duration_parts = duration.split(':');
//                     let hours: i64 = duration_parts.next()?.parse().ok()?;
//                     let minutes: i64 = duration_parts.next()?.parse().ok()?;
//                     let seconds: i64 = duration_parts.next()?.parse().ok()?;

//                     let mut total_seconds = hours * 3600 + minutes * 60 + seconds;
//                     if negative {
//                         total_seconds = -total_seconds;
//                     }
//                     Some(Duration::seconds(total_seconds))
//                 })();
//                 if let Some(new_duration) = new_duration {
//                     notify(RecomputeStopped);
//                     return super::set_time_entry_stopped(time_entry, started + duration)
//                 }
//                 duration.remove()
//             })),
//             duration.inner()
//         ]
//     }

//     #[el]
//     fn started_inputs(time_entry: Var<super::TimeEntry>) -> Column {
//         let (active, started) = time_entry.map(|time_entry| (
//             time_entry.stopped.is_none(),
//             time_entry.started,
//         ));
//         let started_date = el_var(|| started.format("%F").to_string());
//         let started_time = el_var(|| started.format("%X").to_string());
//         column![
//             // 2020-11-03
//             text_input![
//                 active.not().then(|| text_input::on_change(|date| started_date.set(date))),
//                 active.not().then(|| on_blur(|| {
//                     let new_started = (|| {
//                         let date = started_date.map(|date| {
//                             NaiveDate::parse_from_str(&date, "%F").ok() 
//                         })?;
//                         let time = started.time();
//                         Local.from_local_date(&date).and_time(time).single()
//                     })();
//                     if Some(new_started) = new_started {
//                         notify(RecomputeDuration);
//                         return super::set_time_entry_started(time_entry, started);
//                     }
//                     started_date.remove();
//                 })),
//                 started_date.inner(),
//             ],
//             // 14:17:34
//             text_input![
//                 active.not().then(|| text_input::on_change(|time| started_time.set(time))),
//                 active.not().then(|| on_blur(|| {
//                     let new_started = (|| {
//                         let time = started_time.map(|time| {
//                             NaiveTime::parse_from_str(&time, "%X").ok() 
//                         })?;
//                         let date = started.naive_local().date();
//                         Local.from_local_date(&date).and_time(time).single()
//                     })();
//                     if Some(new_started) = new_started {
//                         notify(RecomputeDuration);
//                         return super::set_time_entry_started(time_entry, started);
//                     }
//                     started_time.remove();
//                 })),
//                 started_time.inner(),
//             ],
//         ]
//     }

//     #[el]
//     fn stopped_inputs(time_entry: Var<super::TimeEntry>) -> Column {
//         let (active, stopped) = time_entry.map(|time_entry| (
//             time_entry.stopped.is_none(),
//             time_entry.stopped.unwrap_or_else(Local::now),
//         ));
//         let recompute = listen_ref(|RecomputeStopped| ()).is_some();
//         let stopped_date = el_var_reset(recompute, || stopped.format("%F").to_string());
//         let stopped_time = el_var_reset(recompute, || stopped.format("%X").to_string());
//         column![
//             // 2020-11-03
//             text_input![
//                 active.not().then(|| text_input::on_change(|date| stopped_date.set(date))),
//                 active.not().then(|| on_blur(|| {
//                     let new_stopped = (|| {
//                         let date = stopped_date.map(|date| {
//                             NaiveDate::parse_from_str(&date, "%F").ok() 
//                         })?;
//                         let time = stopped.time();
//                         Local.from_local_date(&date).and_time(time).single()
//                     })();
//                     if Some(new_stopped) = new_stopped {
//                         notify(RecomputeDuration);
//                         return super::set_time_entry_stopped(time_entry, stopped);
//                     }
//                     stopped_date.remove();
//                 })),
//                 stopped_date.inner(),
//             ],
//             // 14:17:34
//             text_input![
//                 active.not().then(|| text_input::on_change(|time| stopped_time.set(time))),
//                 active.not().then(|| on_blur(|| {
//                     let new_stopped = (|| {
//                         let time = stopped_time.map(|time| {
//                             NaiveTime::parse_from_str(&time, "%X").ok() 
//                         })?;
//                         let date = stopped.naive_local().date();
//                         Local.from_local_date(&date).and_time(time).single()
//                     })();
//                     if Some(new_stopped) = new_stopped {
//                         notify(RecomputeDuration);
//                         return super::set_time_entry_stopped(time_entry, stopped);
//                     }
//                     stopped_time.remove();
//                 })),
//                 stopped_time.inner(),
//             ],
//         ]
//     }
    
// }
