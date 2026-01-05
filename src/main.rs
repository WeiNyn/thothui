use std::path::PathBuf;

use gpui::{Size, prelude::FluentBuilder, *};
use gpui_component::{
    button::*,
    checkbox::Checkbox,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    *,
};
use gpui_component_assets::Assets;
use uuid::Uuid;

pub struct TodoItem {
    id: SharedString,
    title: SharedString,
    completed: bool,
}

pub struct DeleteTodo;
impl EventEmitter<DeleteTodo> for TodoItem {}

impl TodoItem {
    pub fn new(title: SharedString) -> Self {
        Self {
            id: Uuid::new_v4().to_string().into(),
            title,
            completed: false,
        }
    }
}

impl Render for TodoItem {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .items_center()
            .justify_between()
            .gap_2()
            .p_2()
            .rounded_sm()
            .flex_grow()
            .m_2()
            .bg(cx.theme().accent)
            .child(
                Checkbox::new(self.id.clone())
                    .checked(self.completed)
                    .w_8()
                    .h_8()
                    .on_click(cx.listener(|this, &e, _, c| {
                        this.completed = e;
                        c.notify();
                    })),
            )
            .child(
                div()
                    .w_4_5()
                    .max_w_4_5()
                    .m_1()
                    .flex_grow()
                    .overflow_hidden()
                    .child(
                        gpui_component::text::TextView::markdown(
                            self.id.clone(),
                            self.title.clone(),
                            window,
                            cx,
                        )
                        .selectable(true),
                    )
                    .when(self.completed, |s| s.line_through()),
            )
            .child(
                Button::new(SharedString::new(format!("delete-{}", self.id.clone())))
                    .icon(IconName::Delete)
                    .w_10()
                    .ghost()
                    .on_click(cx.listener(|_, _, _, c| {
                        c.emit(DeleteTodo);
                    })),
            )
    }
}

struct TodoList {
    items: Vec<Entity<TodoItem>>,
    _selected_index: Option<IndexPath>,
    _subscriptions: Vec<Subscription>,
}

impl TodoList {
    pub fn new() -> Self {
        TodoList {
            items: Vec::new(),
            _selected_index: None,
            _subscriptions: Vec::new(),
        }
    }

    pub fn add_item(&mut self, title: SharedString, cx: &mut Context<TodoList>) -> SharedString {
        let item = cx.new(|_| TodoItem::new(title.clone()));
        let id = item.read(cx).id.clone();

        let subscription = cx.subscribe(&item, |this, e, _, c| {
            this.items.retain(|i| *i != e);
            c.notify();
        });

        self.items.push(item);

        self._subscriptions.push(subscription);
        id
    }
}

struct TodoApp {
    todo_list: Entity<TodoList>,

    input_state: Entity<InputState>,
    editing_text: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl TodoApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            let mut input_state = InputState::new(window, cx)
                .code_editor("markdown")
                .multi_line(true)
                .line_number(true)
                .searchable(true)
                .placeholder("What to do...");
            input_state.set_highlighter("markdown", cx);
            input_state
        });

        let input_subscription = cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => {
                    let value = input_state.read(cx).value();
                    this.editing_text = value.clone();
                    cx.notify();
                }
                InputEvent::PressEnter { secondary } => {
                    if !secondary {
                        // Add item
                        this.todo_list.update(cx, |todo_list, c| {
                            todo_list.add_item(this.editing_text.clone(), c);
                            c.notify();
                        });
                        this.editing_text = SharedString::new("");
                        this.input_state.update(cx, |input_state, cx| {
                            input_state.set_value("", window, cx);
                        });
                        cx.notify();
                    }
                }
                _ => {}
            }
        });

        let todo_list = cx.new(|_| TodoList::new());
        Self {
            todo_list,
            input_state,
            editing_text: SharedString::new(""),
            _subscriptions: vec![input_subscription],
        }
    }
}

impl Render for TodoApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(
                TitleBar::new().h_5().max_h_5().child(
                    h_flex()
                        .w_full()
                        .pr_2()
                        .justify_between()
                        .child("Thoth Note"),
                ),
            )
            .child(
                h_flex()
                    .size_full()
                    .max_h(Pixels::from(window.bounds().bottom().to_f64() - 40.0))
                    .p_2()
                    .justify_between()
                    .child(
                        Input::new(&self.input_state)
                            .content_stretch()
                            .size_full()
                            .overflow_hidden()
                            .m_2()
                            .suffix(Button::new("add").icon(IconName::ArrowRight).on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.todo_list.update(cx, |todo_list, cx| {
                                        todo_list.add_item(this.editing_text.clone(), cx);
                                        cx.notify();
                                    });
                                    this.editing_text = SharedString::new("");
                                    this.input_state.update(cx, |input_state, cx| {
                                        input_state.set_value("", window, cx);
                                    });
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        v_flex()
                            .overflow_y_scrollbar()
                            .h_full()
                            .max_h_full()
                            .relative()
                            .overflow_hidden()
                            .flex_grow()
                            .gap_1_2()
                            .m_2()
                            .p_2()
                            .children(self.todo_list.read(cx).items.clone()),
                    ),
            )
    }
}

fn init(cx: &mut App) {
    if let Err(_err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
        if let Some(theme) = ThemeRegistry::global(cx)
            .themes()
            .get(&SharedString::from("Catppuccin Mocha"))
            .cloned()
        {
            Theme::global_mut(cx).apply_config(&theme);
        }
    }) {
        println!("Failed to load theme")
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);
    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);
        init(cx);

        let bounds = WindowBounds::Windowed(Bounds {
            origin: Point::new(Pixels::from(0.0), Pixels::from(0.0)),
            size: Size::new(Pixels::from(1200.0), Pixels::from(800.0)),
        });

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(bounds),
                    titlebar: Some(TitleBar::title_bar_options()),
                    is_resizable: true,
                    window_min_size: Some(Size::new(Pixels::from(400.0), Pixels::from(200.0))),
                    window_decorations: Some(WindowDecorations::Client),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| TodoApp::new(window, cx));
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
