use zoon::*;
use crate::app;

#[View]
pub fn view() -> View {
    view![
        font::size(14),
        font::family!("Helvetica Neue", "Helvetica", "Arial", font::sans_serif()),
        font::color(hsl(0, 0, 5.1))
        background::color(hsl(0, 0, 96.5)),
        column![
            width!(fill(), minimum(230), maximum(550)),
            center_x(),
            header(),
            main(),
            footer(),
        ]
    ]
}

#[View]
fn header() -> El {
    el![
        region::header(),
        width!(fill()),
        padding!(top(35), bottom(32)),
        el![
            region::h1(),
            center_x(),
            font::size(80),
            font::color(hsl(10.5, 62.8, 44.5)),
            font::extra_light(),
            "todos",
        ],
    ]
}

#[View]
fn main() -> Column {
    column![
        region::section(),
        width!(fill()),
        background::color(hsl(0, 0, 100)),
        border::shadow!(
            shadow::offsetXY(0, 2),
            shadow::size(0),
            shadow::blur(4),
            shadow::color(hsla(0, 0, 0, 20)),
        ),
        border::shadow!(
            shadow::offsetXY(0, 25),
            shadow::size(0),
            shadow::blur(50),
            shadow::color(hsla(0, 0, 0, 10)),
        ),
        row![
            width!(fill()),
            background::color(hsla(0, 0, 0, 0.3)),
            padding!(16),
            border::shadow!(
                shadow::inner(),
                shadow::offsetXY(-2, 1),
                shadow::size(0),
                shadow::color(hsla(0, 0, 0, 3)),
            ),
            app::todos_exist().map_true(toggle_all_checkbox),
            new_todo_title(),
        ],
        app::todos_exist().map_true(|| elements![
            todos(),
            status_bar(),
        ]),
    ]
}

#[View]
fn toggle_all_checkbox() -> Checkbox {
    let checked = app::are_all_completed().inner();
    checkbox![
        checkbox::checked(checked),
        checkbox::on_change(app::check_or_uncheck_all),
        input::label_hidden("Toggle All"),
        el![
            font::color(hsla(0, 0, if checked { 48.4 } else { 91.3 })),
            font::size(22),
            rotate(90),
            "❯",
        ],
    ]
}

#[View]
fn new_todo_title() -> TextInput {
    focused = use_state(|| true);
    text_input![
        focused(focused.inner()),
        on_focused_change(|f| focused.set(f)),
        text_input::on_change(app::set_new_todo_title),
        input::label_hidden("New Todo Title"),
        placeholder![
            font::italic(),
            font::light(),
            font::color(hsla(0, 0, 0, 40)),
            placeholder::text("what needs to be done?"),
        ],
        app::new_todo_title().inner(),
    ]
}

#[View]
fn todos() -> Column {
    column![
        app::filtered_todos().iter().map(todo)
    ]
}

fn active_todo_checkbox_icon() -> &'static str {
    "data:image/svg+xml;utf8,%3Csvg%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%22-10%20-18%20100%20135%22%3E%3Ccircle%20cx%3D%2250%22%20cy%3D%2250%22%20r%3D%2250%22%20fill%3D%22none%22%20stroke%3D%22%23ededed%22%20stroke-width%3D%223%22/%3E%3C/svg%3E"
}

fn completed_todo_checkbox_icon() -> &'static str {
    "data:image/svg+xml;utf8,%3Csvg%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%22-10%20-18%20100%20135%22%3E%3Ccircle%20cx%3D%2250%22%20cy%3D%2250%22%20r%3D%2250%22%20fill%3D%22none%22%20stroke%3D%22%23bddad5%22%20stroke-width%3D%223%22/%3E%3Cpath%20fill%3D%22%235dc2af%22%20d%3D%22M72%2025L42%2071%2027%2056l-4%204%2020%2020%2034-52z%22/%3E%3C/svg%3E"
}

#[View]
fn todo(todo: Model<app::Todo>) -> Row {
    let selected_todo_id = app::selected_todo().map(|t| t.map(|t| t.id));

    let selected = Some(todo.map(|t| t.id)) == selected_todo_id;
    let completed = todo.map(|t| t.completed);

    let checkbox_id = use_state(ElementId::new);
    let row_hovered = use_state(|| false);

    row![
        font::size(24),
        padding!(15),
        spacing(10),
        on_hovered_change(|h| row_hovered.set(h)),
        checkbox![
            id(checkbox_id.inner()),
            checkbox::checked(completed),
            checkbox::on_change(|_| app::toggle_todo(todo)),
            el![
                background::image(if completed {
                    completed_todo_checkbox_icon()
                } else {
                    active_todo_checkbox_icon()
                }),
            ],
        ],
        if selected {
            selected_todo_title().into_element()
        } else {
            label![
                label::for_input(checkbox_id.inner()),
                checked.map_true(font::strike),
                font::regular(),
                font::color(hsl(0, 0, 32.7)),
                on_double_click(|| select_todo(Some(todo))),
                todo.map(|t| t.title.clone()),
            ].into_element()
        },
        row_hovered.inner().map_true(|| remove_todo_button(todo)),
    ]
}

#[View]
fn selected_todo_title() -> TextInput {
    let selected_todo = app::selected_todo().inner().expect("selected todo");
    let focused = use_state(|| true);
    text_input![
        width!(fill()),
        paddingXY(16, 12),
        border::solid(),
        border::width!(1),
        border::color(hsl(0, 0, 63.2)),
        border::shadow!(
            shadow::inner(),
            shadow::offsetXY(-1, 5),
            shadow::size(0),
            shadow::color(hsla(0, 0, 0, 20)),
        ),
        focused(focused.inner()),
        on_focused_change(|f| {
            focused.set(f);
            if f { app::save_selected_todo() };
        }),
        on_key_down(|key| {
            match key {
                ESCAPE_KEY =>  app::select_todo(None),
                ENTER_KEY => app::save_selected_todo(),
            }
        }),
        text_input::on_change(app::set_selected_todo_title),
        selected_todo.title,
    ]
}

#[View]
fn remove_todo_button(todo: Model<app::Todo>) -> Button {
    let hovered = use_state(|| false);
    button![
        size::width!(20),
        size::height!(20),
        font::size(30),
        font::color(hsl(12.2, 34.7, 68.2)),
        on_hovered_change(|h| row_hovered.set(h)),
        font::color(if hovered().inner() { hsl(10.5, 37.7, 48.8) } else { hsl(12.2, 34.7, 68.2) }),
        button::on_press(|| app::remove_todo(todo)),
        "×",
    ]
}

#[View]
fn status_bar() -> Row {
    row![
        active_items_count(),
        filters(),
        app::completed_exist().map_true(clear_completed_button),
    ]
}

#[View]
fn active_items_count() -> Paragraph {
    let active_count = app::active_count().inner();
    paragraph![
        el![
            font::bold(),
            active_count,
        ],
        format!(" item{} left", if active_count == 1 { "" } else { "s" }),
    ]
}

#[View]
fn filters() -> Row {
    row![
        app::filters().iter().map(filter)  
    ]
}

#[View]
fn filter(filter: app::Filter) -> Button {
    let selected = app::selected_filter().inner() == filter;
    let (title, route) = match filter {
        app::Filter::All => ("All", app::Route::root()),
        app::Filter::Active => ("Active", app::Route::active()),
        app::Filter::Completed => ("Completed", app::Route::completed()),
    };
    let border_alpha = if selected { 20 } else if hovered { 10 } else { 10 };
    button![
        paddingXY(7, 3),
        border::solid(),
        border::width!(1),
        border::color(hsla(12.2, 72.8, 40.2, border_alpha),
        button::on_press(|| app::set_route(route)),
        title,
    ]
}

#[View]
fn clear_completed_button() -> Button {
    let hovered = use_state(|| false);
    button![
        on_hovered_change(|h| row_hovered.set(h)),
        hovered.inner().map_true(font::underline),
        button::on_press(app::remove_completed),
        "Clear completed",
    ]
}

#[View]
fn footer() -> Column {
    column![
        paragraph![
            "Double-click to edit a todo",
        ],
        paragraph![
            "Created by ",
            link![
                link::new_tab(),
                link::url("https://github.com/MartinKavik"),
                "Martin Kavík",
            ],
        ],
        paragraph![
            "Part of ",
            link![
                link::new_tab(),
                link::url("http://todomvc.com"),
                "TodoMVC",
            ],
        ],
    ]
}
