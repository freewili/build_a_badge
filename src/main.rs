// main.rs
use iced::widget::{button, column, container, row, text, Space, image, radio, text_input};
use iced::{
    Alignment, Element, Length, Settings, Theme, Color, Size, Border,
    Application, Command, Subscription, executor,
    ContentFit,
};
use iced::window;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

// Explicitly import necessary types and traits for Iced 0.12.1
use iced::widget::button::{Appearance as ButtonAppearance, StyleSheet as ButtonStyleSheet};
use iced::widget::container::{Appearance as ContainerAppearance, StyleSheet as ContainerStyleSheet};
// --- Constants and Statics ---
static YELLOW: LazyLock<Color> = LazyLock::new(|| Color::from_rgb8(255, 191, 0));
static BLUE_TEXT: LazyLock<Color> = LazyLock::new(|| Color::from_rgb8(0, 85, 150));

// --- Static handles for your images ---
static BADGE_PLACEHOLDER_IMAGE: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/badge_placeholder.png").to_vec())
});
static STUFF_ME_ICON: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/stuff_me_icon.png").to_vec())
});
static LIGHT_ME_ICON: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/light_me_icon.png").to_vec())
});
static PLACE_ME_ICON: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/place_me_icon.png").to_vec())
});
static APP_LOGO_IMAGE: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/logo.png").to_vec())
});
static NAME_ME_IMAGE: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/badge_placeholder.png").to_vec())
});

static DEFAULT_PLACEHOLDER_IMAGE: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/badge_placeholder.png").to_vec())
});

// --- NEW: Static handles for user-selectable images ---
static DEFCON_LOGO_IMAGE: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/defcon_logo.png").to_vec())
});
static DOGE_IMAGE: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/doge.png").to_vec())
});
static ELON_IMAGE: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/puppy.png").to_vec())
});
static VEGAS_IMAGE: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/vegas.png").to_vec())
});

const HEADING_SIZE: u16 = 30;
const BODY_SIZE: u16 = 18;
const BUTTON_TEXT_SIZE: u16 = 24;

// Animation constants
const TRANSITION_DURATION_MILLIS: u64 = 300; // Total duration for fade in/out
const ANIMATION_TICK_MILLIS: u64 = 16; // Roughly 60 FPS

// --- Application State and Messages ---
#[derive(Debug, Clone, Copy, PartialEq)]
enum AppScreen {
    Welcome,
    CustomizeBadge,
    CustomizeLeds,
    NameBadge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LedMode {
    Rainbow,
    BlueChase,
    GreenDot,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppScreenTransition {
    Idle,
    FadingOut(AppScreen),
    FadingIn,
}


struct BuildABadgeApp {
    current_screen: AppScreen,
    selected_customize_image: Option<image::Handle>,
    selected_led_mode: Option<LedMode>,
    badge_name: String,
    
    // Animation state
    transition: AppScreenTransition,
    current_opacity: f32,
    transition_start_time: Instant,
}

#[derive(Debug, Clone)]
enum Message {
    NavigateTo(AppScreen),
    SelectCustomizeImage(image::Handle),
    SelectLedMode(LedMode),
    BadgeNameChanged(String),
    
    // Animation messages
    Tick(Instant),
}

pub fn main() -> iced::Result {
    let settings = Settings {
        window: window::Settings {
            size: Size::new(1000.0, 700.0),
            min_size: Some(Size::new(800.0, 500.0)),
            ..window::Settings::default()
        },
        antialiasing: true,
        ..Settings::default()
    };
    BuildABadgeApp::run(settings)
}


// --- BuildABadgeApp Implementation - Core Application Logic ---
impl Application for BuildABadgeApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        // The initial application state
        let app_state = Self {
            current_screen: AppScreen::Welcome,
            selected_customize_image: None,
            selected_led_mode: None,
            badge_name: String::new(),
            
            transition: AppScreenTransition::FadingIn,
            current_opacity: 0.0,
            transition_start_time: Instant::now(),
        };

        (
            app_state,
            Command::none() // No initial commands, subscriptions are in `subscription()`
        )
    }

    fn title(&self) -> String {
        String::from("Build-A-Badge")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NavigateTo(screen) => {
                if self.current_screen != screen && self.transition == AppScreenTransition::Idle {
                    self.transition = AppScreenTransition::FadingOut(screen);
                    self.transition_start_time = Instant::now();
                }
            }
            Message::SelectCustomizeImage(handle) => {
                self.selected_customize_image = Some(handle);
            }
            Message::SelectLedMode(mode) => {
                self.selected_led_mode = Some(mode);
            }
            Message::BadgeNameChanged(name) => {
                self.badge_name = name;
            }

            Message::Tick(now) => {
                let elapsed_transition = (now - self.transition_start_time).as_millis() as u64;
                
                match self.transition {
                    AppScreenTransition::FadingOut(target_screen) => {
                        let progress = (elapsed_transition as f32 / TRANSITION_DURATION_MILLIS as f32).min(1.0);
                        self.current_opacity = 1.0 - progress;

                        if progress >= 1.0 {
                            self.current_screen = target_screen;
                            self.transition = AppScreenTransition::FadingIn;
                            self.transition_start_time = Instant::now();
                        }
                    },
                    AppScreenTransition::FadingIn => {
                        let progress = (elapsed_transition as f32 / TRANSITION_DURATION_MILLIS as f32).min(1.0);
                        self.current_opacity = progress;

                        if progress >= 1.0 {
                            self.transition = AppScreenTransition::Idle;
                            self.current_opacity = 1.0;
                        }
                    },
                    AppScreenTransition::Idle => {}
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(ANIMATION_TICK_MILLIS))
            .map(Message::Tick)
    }

    fn view(&self) -> Element<Message> {
        let current_screen_element = match self.current_screen {
            AppScreen::Welcome => self.render_welcome_screen(),
            AppScreen::CustomizeBadge => self.render_customize_badge_screen(),
            AppScreen::CustomizeLeds => self.render_customize_leds_screen(),
            AppScreen::NameBadge => self.render_name_badge_screen(),
        };

        let animated_content = container(current_screen_element)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(theme_fn_container(ScreenTransitionStyle(self.current_opacity)));
        
        column![
            animated_content
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}


// --- BuildABadgeApp Implementation - Custom Methods (Rendering and Helpers) ---
impl BuildABadgeApp {
    fn render_welcome_screen(&self) -> Element<Message> {
    let start_button = button(
        text("Start")
        .size(BUTTON_TEXT_SIZE)
        .horizontal_alignment(iced::alignment::Horizontal::Center)
    )
    .on_press(Message::NavigateTo(AppScreen::CustomizeBadge))
    .padding([10, 40])
    .style(theme_fn(YellowButtonStyle));

    // Create a container for the logo with a fixed height to control its size
    let app_logo_container = container(
        image(APP_LOGO_IMAGE.clone())
            .width(Length::Fixed(400.0)) // Set a fixed width
            .height(Length::Fixed(400.0)) // Set a fixed height
            .content_fit(ContentFit::ScaleDown) // Ensure the image scales down
    )
    .width(Length::Fill) // Make the container fill the width
    .height(Length::Shrink) // Shrink height to fit the content
    .center_x()
    .center_y();

    column![
        Space::new(Length::Shrink, Length::Fixed(50.0)),
        app_logo_container,
        Space::new(Length::Shrink, Length::Fixed(20.0)),
        container::<_, Theme, iced::Renderer>(
            text("Welcome to the Build-A-Badge Workshop, where creativity and fun come alive for all! Dive into a vibrant space that sparks your imagination, encourages discovery, and empowers you to craft a unique DEFCON badge—along with memories to cherish forever.")
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .size(BODY_SIZE)
        )
        .padding([0, 100]),
        Space::new(Length::Shrink, Length::Fixed(20.0)),
        container::<_, Theme, iced::Renderer>(
            text("What’s in store? Get ready to hack your ICS Village Badge with a hands-on, interactive experience! You’ll create custom script and design your very own application, guided every step of the way by a friendly ICS Village Badge Builder associate.")
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .size(BODY_SIZE)
        )
        .padding([0, 100]),
        Space::new(Length::Shrink, Length::Fixed(40.0)),
        start_button
    ]
    .spacing(20)
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

    fn render_customize_badge_screen(&self) -> Element<Message> {
        let display_image_handle = self.selected_customize_image.as_ref().map_or_else(|| BADGE_PLACEHOLDER_IMAGE.clone(), |h| h.clone());

        let user_image_widget = image(display_image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::ScaleDown);

        let user_image_container = container(user_image_widget)
            .width(Length::FillPortion(2))
            .height(Length::FillPortion(2))
            .center_x()
            .center_y()
            .style(theme_fn_container(UserImageBorderStyle));

        let choose_image_text = text("Choose a picture for your badge:")
            .size(BODY_SIZE)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .width(Length::Fill);

        let images_to_select = [
            &DEFCON_LOGO_IMAGE,
            &DOGE_IMAGE,
            &ELON_IMAGE,
            &VEGAS_IMAGE,
        ];

        let mut image_selection_row = row![]
            .spacing(10)
            .align_items(Alignment::Center)
            .width(Length::Fill);

        for img_handle_lazy in images_to_select.iter() {
            let img_handle_to_compare = (**img_handle_lazy).clone();
            let is_selected = self.selected_customize_image.as_ref().map_or(false, |selected| {
                selected.eq(&img_handle_to_compare)
            });

            let button_style = if is_selected {
                theme_fn(SelectedBadgeStyle)
            } else {
                theme_fn(DefaultBadgeStyle)
            };

            let image_button_content: iced::widget::Image<image::Handle> = image((**img_handle_lazy).clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::ScaleDown);

            let image_button: iced::widget::Button<'_, Message, Theme, iced::Renderer> = button(image_button_content)
                .on_press(Message::SelectCustomizeImage((**img_handle_lazy).clone()))
                .padding(5)
                .style(button_style);

            image_selection_row = image_selection_row.push(
                container(image_button)
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(160.0))
                    .center_x()
                    .center_y()
            );
        }

        let back_button = button(text("Back").size(BODY_SIZE))
            .on_press(Message::NavigateTo(AppScreen::Welcome))
            .padding([5,20])
            .style(theme_fn(YellowButtonStyle));

        let finish_button_enabled = self.selected_customize_image.is_some();
        let finish_button_style = if finish_button_enabled {
            theme_fn(YellowButtonStyle)
        } else {
            theme_fn(DisabledButtonStyle)
        };
        let finish_button_message = if finish_button_enabled {
            Message::NavigateTo(AppScreen::CustomizeLeds)
        } else {
            Message::SelectCustomizeImage(DEFAULT_PLACEHOLDER_IMAGE.clone())
        };

        let finish_button = button(text("Next").size(BODY_SIZE))
            .on_press_maybe(if finish_button_enabled { Some(finish_button_message) } else { None })
            .padding([5,20])
            .style(finish_button_style);

        let bottom_buttons = row![
            back_button,
            Space::with_width(Length::Fill),
            finish_button,
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fill);


        column![
            Space::with_height(Length::Fixed(50.0)),
            image(STUFF_ME_ICON.clone())
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0)),
            text("Stuff Me")
                .size(HEADING_SIZE - 5)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            text("Customize Badge with your picture")
                .size(HEADING_SIZE - 5)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::new(Length::Shrink, Length::Fixed(10.0)),
            row![
                user_image_container,
                column![
                    choose_image_text,
                    Space::new(Length::Shrink, Length::Fixed(20.0)),
                    image_selection_row,
                ]
                .width(Length::FillPortion(3))
                .align_items(Alignment::Center)
                .spacing(10)
            ]
            .spacing(20)
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .height(Length::FillPortion(1)),
            Space::new(Length::Shrink, Length::Fixed(20.0)),
            bottom_buttons
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn render_customize_leds_screen(&self) -> Element<Message> {
        let back_button = button(text("Back").size(BODY_SIZE))
            .on_press(Message::NavigateTo(AppScreen::CustomizeBadge))
            .padding([5,20])
            .style(theme_fn(YellowButtonStyle));
        
        let confirm_button = button(text("Confirm").size(BODY_SIZE))
            .on_press(Message::NavigateTo(AppScreen::NameBadge))
            .padding([5,20])
            .style(theme_fn(YellowButtonStyle));

        let bottom_buttons = row![
            back_button,
            Space::with_width(Length::Fill),
            confirm_button,
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fill);

        let modes = [
            LedMode::Rainbow, 
            LedMode::BlueChase, 
            LedMode::GreenDot, 
            LedMode::Manual
        ];

        let radio_row = modes.iter().fold(
            row!().spacing(20).align_items(Alignment::Center),
            |row, mode| {
                row.push(radio(
                    format!("{:?}", mode),
                    *mode,
                    self.selected_led_mode,
                    Message::SelectLedMode,
                )
                .size(BODY_SIZE)
                .spacing(5))
            },
        );

        column![
            Space::with_height(Length::Fixed(50.0)),
            image(LIGHT_ME_ICON.clone())
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0)),
            text("Light Me")
                .size(HEADING_SIZE)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            container(
                text("Customize your Badge LEDs")
                .size(HEADING_SIZE)
                .style(iced::theme::Text::Color(*BLUE_TEXT))
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(Length::Fill)
            )
            .padding([0, 50]),
            Space::new(Length::Shrink, Length::Fixed(40.0)),
            radio_row,
            Space::with_height(Length::Fill),
            bottom_buttons,
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    fn render_name_badge_screen(&self) -> Element<Message> {
        let back_button = button(text("Back").size(BODY_SIZE))
            .on_press(Message::NavigateTo(AppScreen::CustomizeLeds))
            .padding([5,20])
            .style(theme_fn(YellowButtonStyle));

        let submit_button = button(text("Submit").size(BODY_SIZE))
            .on_press(Message::NavigateTo(AppScreen::Welcome))
            .padding([5,20])
            .style(theme_fn(YellowButtonStyle));

        let bottom_buttons = row![
            back_button,
            Space::with_width(Length::Fill),
            submit_button,
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fill);

        column![
            Space::with_height(Length::Fixed(50.0)),
            image(PLACE_ME_ICON.clone())
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0)),
            text("Place Me")
                .size(HEADING_SIZE)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::new(Length::Fill, Length::Fill),
            row![
                container(image(NAME_ME_IMAGE.clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(ContentFit::ScaleDown))
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1))
                    .align_x(iced::alignment::Horizontal::Center)
                    .align_y(iced::alignment::Vertical::Center),
                container(column![
                    text("Type Your Badge Name Below")
                        .size(HEADING_SIZE)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                        .width(Length::Fill),
                    Space::new(Length::Shrink, Length::Fixed(20.0)),
                    text_input::<_, Theme, iced::Renderer>("Enter name...", &self.badge_name)
                        .on_input(Message::BadgeNameChanged)
                        .padding(10)
                        .size(BODY_SIZE)
                        .width(Length::Fill),
                ])
                .width(Length::FillPortion(1))
                .height(Length::Shrink)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            ]
            .width(Length::Fill)
            .height(Length::FillPortion(2))
            .align_items(Alignment::Center)
            .spacing(20),
            Space::new(Length::Fill, Length::Fill),
            bottom_buttons,
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}


// --- Custom Styles (unchanged) ---
struct YellowButtonStyle;
impl ButtonStyleSheet for YellowButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some((*YELLOW).into()),
            text_color: Color::BLACK,
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let active = self.active(style);
        ButtonAppearance { background: Some(Color { a: 0.8, ..*YELLOW }.into()), ..active }
    }
}

struct DisabledButtonStyle;
impl ButtonStyleSheet for DisabledButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(Color::from_rgb(0.7, 0.7, 0.7).into()),
            text_color: Color::from_rgb(0.4, 0.4, 0.4),
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

struct TopBannerBackgroundStyle;
impl ContainerStyleSheet for TopBannerBackgroundStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> ContainerAppearance {
        ContainerAppearance {
            background: Some((*YELLOW).into()),
            ..Default::default()
        }
    }
}

struct DefaultTopBannerItemStyle;
impl ButtonStyleSheet for DefaultTopBannerItemStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(Color::TRANSPARENT.into()),
            text_color: Color::BLACK,
            border: Border { color: Color::BLACK, width: 1.0, radius: 2.0.into() },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(Color{r:0.0,g:0.0,b:0.0,a:0.1}.into()),
            ..self.active(style)
        }
    }
}

struct SelectedTopBannerItemStyle;
impl ButtonStyleSheet for SelectedTopBannerItemStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(Color {a: 0.3, ..*YELLOW}.into()),
            text_color: Color::BLACK,
            border: Border { color: Color::BLACK, width: 2.0, radius: 3.0.into() },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        self.active(style)
    }
}

struct DefaultBadgeStyle;
impl ButtonStyleSheet for DefaultBadgeStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(Color::WHITE.into()),
            text_color: Color::BLACK,
            border: Border { color: Color::BLACK, width: 1.0, radius: 8.0.into() },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let active = self.active(style);
        ButtonAppearance {
            background: Some(Color::from_rgb(0.95, 0.95, 0.95).into()),
            ..active
        }
    }
}

struct SelectedBadgeStyle;
impl ButtonStyleSheet for SelectedBadgeStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(Color {a: 0.3, ..*YELLOW}.into()),
            text_color: Color::BLACK,
            border: Border { color: *YELLOW, width: 2.0, radius: 8.0.into() },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        self.active(style)
    }
}

struct UserImageBorderStyle;
impl ContainerStyleSheet for UserImageBorderStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> ContainerAppearance {
        ContainerAppearance {
            background: None,
            border: Border {
                color: Color::BLACK,
                width: 2.0,
                radius: 92.0.into(),
            },
            ..Default::default()
        }
    }
}

struct ScreenTransitionStyle(f32);
impl ContainerStyleSheet for ScreenTransitionStyle {
    type Style = Theme;
    fn appearance(&self, style: &Self::Style) -> ContainerAppearance {
        let mut appearance = style.appearance(&iced::theme::Container::Box); 
        
        if let Some(mut text_color) = appearance.text_color {
            text_color.a *= self.0;
            appearance.text_color = Some(text_color);
        }
        
        if let Some(mut background) = appearance.background {
            match &mut background {
                iced::Background::Color(color) => {
                    color.a *= self.0;
                    appearance.background = Some(background);
                },
                _ => {}
            }
        }

        appearance
    }
}

struct DialogBackgroundStyle;
impl ContainerStyleSheet for DialogBackgroundStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> ContainerAppearance {
        ContainerAppearance {
            background: Some(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }.into()),
            ..Default::default()
        }
    }
}

fn theme_fn<T>(style: T) -> iced::theme::Button
where
    T: ButtonStyleSheet<Style = Theme> + 'static,
    T::Style: Sized,
{
    iced::theme::Button::Custom(Box::new(style))
}

fn theme_fn_container<T>(style: T) -> iced::theme::Container
where
    T: ContainerStyleSheet<Style = Theme> + 'static,
    T::Style: Sized,
{
    iced::theme::Container::Custom(Box::new(style))
}