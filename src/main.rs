// main.rs
use iced::widget::{
    Space, button, column, container, image, progress_bar, radio, row, text, text_input,
};
use iced::window;
use iced::{
    Alignment, Application, Border, Color, Command, ContentFit, Element, Length, Settings, Size,
    Subscription, Theme, executor, event, mouse, keyboard,
};
use std::fs;
use std::sync::LazyLock;
use std::time::Duration;

// Explicitly import necessary types and traits for Iced 0.12.1
use iced::widget::button::{Appearance as ButtonAppearance, StyleSheet as ButtonStyleSheet};
use iced::widget::container::{
    Appearance as ContainerAppearance, StyleSheet as ContainerStyleSheet,
};
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
static APP_LOGO_IMAGE: LazyLock<image::Handle> =
    LazyLock::new(|| image::Handle::from_memory(include_bytes!("../assets/logo.png").to_vec()));
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
static DOGE_IMAGE: LazyLock<image::Handle> =
    LazyLock::new(|| image::Handle::from_memory(include_bytes!("../assets/doge.png").to_vec()));
static PUPPY_IMAGE: LazyLock<image::Handle> =
    LazyLock::new(|| image::Handle::from_memory(include_bytes!("../assets/puppy.png").to_vec()));
static PIP_BOY_IMAGE: LazyLock<image::Handle> =
    LazyLock::new(|| image::Handle::from_memory(include_bytes!("../assets/pip_boy.jpg").to_vec()));
static VEGAS_IMAGE: LazyLock<image::Handle> =
    LazyLock::new(|| image::Handle::from_memory(include_bytes!("../assets/vegas.png").to_vec()));

const HEADING_SIZE: u16 = 30;
const BODY_SIZE: u16 = 18;
const BUTTON_TEXT_SIZE: u16 = 24;

// No animation constants needed for instant transitions

// Text input ID for focus management
const BADGE_NAME_INPUT_ID: &str = "badge_name_input";

// --- Application State and Messages ---
#[derive(Debug, Clone, Copy, PartialEq)]
enum AppScreen {
    Welcome,
    CustomizeBadge,
    CustomizeLeds,
    NameBadge,
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LedMode {
    Manual,
    Rainbow,
    Snowstorm,
    RedChase,
    RainbowChase,
    BlueChase,
    GreenDot,
    BlueDot,
    BlueSin,
    WhiteFade,
    BarGraph,
    Zylon,
    Audio,
    Accel,
}

impl LedMode {
    fn display_name(&self) -> &'static str {
        match self {
            LedMode::Manual => "Manual",
            LedMode::Rainbow => "Rainbow",
            LedMode::Snowstorm => "Snowstorm",
            LedMode::RedChase => "Red Chase",
            LedMode::RainbowChase => "Rainbow Chase",
            LedMode::BlueChase => "Blue Chase",
            LedMode::GreenDot => "Green Dot",
            LedMode::BlueDot => "Blue Dot",
            LedMode::BlueSin => "Blue Sin",
            LedMode::WhiteFade => "White Fade",
            LedMode::BarGraph => "Bar Graph",
            LedMode::Zylon => "Zylon",
            LedMode::Audio => "Audio",
            LedMode::Accel => "Accelerometer",
        }
    }

    fn as_integer(&self) -> u8 {
        match self {
            LedMode::Manual => 0,
            LedMode::Rainbow => 1,
            LedMode::Snowstorm => 2,
            LedMode::RedChase => 3,
            LedMode::RainbowChase => 4,
            LedMode::BlueChase => 5,
            LedMode::GreenDot => 6,
            LedMode::BlueDot => 7,
            LedMode::BlueSin => 8,
            LedMode::WhiteFade => 9,
            LedMode::BarGraph => 10,
            LedMode::Zylon => 11,
            LedMode::Audio => 12,
            LedMode::Accel => 13,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppScreenTransition {
    Idle,
    // Removed FadingOut and FadingIn since we have instant transitions
}

struct BuildABadgeApp {
    current_screen: AppScreen,
    selected_customize_image: Option<image::Handle>,
    selected_led_mode: Option<LedMode>,
    badge_name: String,

    // Configuration state
    is_configuring: bool,
    configuration_progress: f32,
    configuration_status: String,
    configuration_error: Option<String>,
    configuration_console_output: String,

    // Animation state (simplified for instant transitions)
    transition: AppScreenTransition,
    current_opacity: f32,
}

#[derive(Debug, Clone)]
enum Message {
    NavigateTo(AppScreen),
    SelectCustomizeImage(image::Handle),
    SelectLedMode(LedMode),
    BadgeNameChanged(String),
    StartConfiguration,
    ConfigurationStepUpdate(String, f32), // step description, progress (0.0-1.0)
    ConfigurationComplete(Result<String, String>),
    MouseButtonPressed(iced::mouse::Button),
    KeyPressed(iced::keyboard::Key),
}

pub fn main() -> iced::Result {
    let settings = Settings {
        window: window::Settings {
            size: Size::new(1200.0, 900.0),
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
            selected_led_mode: Some(LedMode::Accel), // Default to Accelerometer
            badge_name: String::new(),

            is_configuring: false,
            configuration_progress: 0.0,
            configuration_status: String::new(),
            configuration_error: None,
            configuration_console_output: String::new(),

            transition: AppScreenTransition::Idle,
            current_opacity: 1.0,
        };

        (
            app_state,
            Command::none(), // No initial commands, subscriptions are in `subscription()`
        )
    }

    fn title(&self) -> String {
        String::from("Build-A-Badge")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NavigateTo(screen) => {
                if self.current_screen != screen {
                    // Instant navigation - no transitions
                    self.current_screen = screen;
                    self.transition = AppScreenTransition::Idle;
                    self.current_opacity = 1.0;

                    // Clear configuration status when navigating away from summary
                    if screen != AppScreen::Summary {
                        self.configuration_status = String::new();
                        self.configuration_error = None;
                        self.configuration_progress = 0.0;
                        self.configuration_console_output = String::new();
                    }

                    // Focus the text input when navigating to the name badge screen
                    if screen == AppScreen::NameBadge {
                        return Command::batch([
                            text_input::focus(text_input::Id::new(BADGE_NAME_INPUT_ID))
                        ]);
                    }
                }
            }
            Message::SelectCustomizeImage(handle) => {
                self.selected_customize_image = Some(handle);
            }
            Message::SelectLedMode(mode) => {
                self.selected_led_mode = Some(mode);
            }
            Message::BadgeNameChanged(name) => {
                // Filter to alphanumeric characters only and limit to 23 characters
                let filtered_name: String = name.chars().filter(|c| c.is_alphanumeric()).collect();

                if filtered_name.len() <= 23 {
                    self.badge_name = filtered_name;
                }
            }
            Message::StartConfiguration => {
                self.is_configuring = true;
                self.configuration_progress = 0.0;
                self.configuration_status = "Starting configuration...".to_string();
                self.configuration_error = None;
                self.configuration_console_output = String::new();
            }
            Message::ConfigurationStepUpdate(step_description, progress) => {
                self.configuration_status = step_description.clone();
                self.configuration_progress = progress;
                
                // Append to console output for live updates
                if !self.configuration_console_output.is_empty() {
                    self.configuration_console_output.push_str("\n");
                }
                self.configuration_console_output.push_str(&step_description);
                
                // ConfigurationStepUpdate is only for progress updates, errors use ConfigurationComplete
            }
            Message::ConfigurationComplete(result) => {
                self.is_configuring = false;
                self.configuration_progress = 1.0;

                match result {
                    Ok(message) => {
                        println!("Configuration successful: {}", message);
                        self.configuration_status = "Configuration successful!".to_string();
                        self.configuration_error = None;
                        // Keep the existing console output and append success message
                        if !self.configuration_console_output.is_empty() {
                            self.configuration_console_output.push_str("\n");
                        }
                        self.configuration_console_output.push_str(&message);
                    }
                    Err(error) => {
                        println!("Configuration failed: {}", error);
                        self.configuration_status = "Configuration failed".to_string();
                        self.configuration_error = Some(error.clone());
                        // Keep the existing console output and append error message
                        if !self.configuration_console_output.is_empty() {
                            self.configuration_console_output.push_str("\n");
                        }
                        self.configuration_console_output.push_str(&error);
                    }
                }
            }
            Message::MouseButtonPressed(button) => {
                // Handle mouse back and forward buttons
                match button {
                    mouse::Button::Back => {
                        // Navigate backwards based on current screen
                        let previous_screen = match self.current_screen {
                            AppScreen::Welcome => None,
                            AppScreen::CustomizeBadge => Some(AppScreen::Welcome),
                            AppScreen::CustomizeLeds => Some(AppScreen::CustomizeBadge),
                            AppScreen::NameBadge => Some(AppScreen::CustomizeLeds),
                            AppScreen::Summary => Some(AppScreen::NameBadge),
                        };
                        
                        if let Some(screen) = previous_screen {
                            return self.update(Message::NavigateTo(screen));
                        }
                    }
                    mouse::Button::Forward => {
                        // Navigate forwards based on current screen
                        let next_screen = match self.current_screen {
                            AppScreen::Welcome => Some(AppScreen::CustomizeBadge),
                            AppScreen::CustomizeBadge => Some(AppScreen::CustomizeLeds),
                            AppScreen::CustomizeLeds => Some(AppScreen::NameBadge),
                            AppScreen::NameBadge => Some(AppScreen::Summary),
                            AppScreen::Summary => None,
                        };
                        
                        if let Some(screen) = next_screen {
                            return self.update(Message::NavigateTo(screen));
                        }
                    }
                    _ => {} // Ignore other mouse buttons
                }
            }
            Message::KeyPressed(key) => {
                // Handle Enter key on welcome screen to trigger start button
                if let iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) = key {
                    if self.current_screen == AppScreen::Welcome {
                        return self.update(Message::NavigateTo(AppScreen::CustomizeBadge));
                    }
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();
        
        // Add mouse button subscription for navigation
        subscriptions.push(
            event::listen_with(|event, _status| {
                match event {
                    event::Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                        Some(Message::MouseButtonPressed(button))
                    }
                    event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                        Some(Message::KeyPressed(key))
                    }
                    _ => None,
                }
            })
        );
        
        // Add configuration subscription if configuring
        if self.is_configuring {
            let config_subscription = configuration_subscription(
                self.selected_customize_image.clone(),
                self.selected_led_mode,
                self.badge_name.clone(),
            );
            subscriptions.push(config_subscription);
        }
        
        Subscription::batch(subscriptions)
    }

    fn view(&self) -> Element<Message> {
        let current_screen_element = match self.current_screen {
            AppScreen::Welcome => self.render_welcome_screen(),
            AppScreen::CustomizeBadge => self.render_customize_badge_screen(),
            AppScreen::CustomizeLeds => self.render_customize_leds_screen(),
            AppScreen::NameBadge => self.render_name_badge_screen(),
            AppScreen::Summary => self.render_summary_screen(),
        };

        // No transitions - just show the current screen directly
        container(current_screen_element)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

// --- BuildABadgeApp Implementation - Custom Methods (Rendering and Helpers) ---
impl BuildABadgeApp {
    fn render_welcome_screen(&self) -> Element<Message> {
        let start_button = button(
            text("Start")
                .size(BUTTON_TEXT_SIZE)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        )
        .on_press(Message::NavigateTo(AppScreen::CustomizeBadge))
        .padding([10, 40])
        .style(theme_fn(YellowButtonStyle));

        // Create a container for the logo with a smaller, responsive height
        let app_logo_container = container(
            image(APP_LOGO_IMAGE.clone())
                .width(Length::Fixed(300.0)) // Reduced from 400 to 300
                .height(Length::Fixed(250.0)) // Reduced from 400 to 250
                .content_fit(ContentFit::ScaleDown), // Ensure the image scales down
        )
        .width(Length::Fill) // Make the container fill the width
        .height(Length::Shrink) // Shrink height to fit the content
        .center_x()
        .center_y();

        column![
        Space::new(Length::Shrink, Length::Fixed(20.0)), // Reduced from 50
        app_logo_container,
        Space::new(Length::Shrink, Length::Fixed(15.0)), // Reduced from 20
        container::<_, Theme, iced::Renderer>(
            text("Welcome to the Build-A-Badge Workshop, where creativity and fun come alive for all! Dive into a vibrant space that sparks your imagination, encourages discovery, and empowers you to craft a unique DEFCON badge—along with memories to cherish forever.")
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .size(BODY_SIZE)
        )
        .padding([0, 80]), // Reduced horizontal padding from 100 to 80
        Space::new(Length::Shrink, Length::Fixed(10.0)), // Reduced from 20
        container::<_, Theme, iced::Renderer>(
            text("What's in store? Get ready to hack your ICS Village Badge with a hands-on, interactive experience! You'll create custom script and design your very own application, guided every step of the way by a friendly ICS Village Badge Builder associate.")
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .size(BODY_SIZE)
        )
        .padding([0, 80]), // Reduced horizontal padding from 100 to 80
        Space::new(Length::Shrink, Length::Fixed(20.0)), // Reduced from 40
        start_button
    ]
    .spacing(15) // Reduced from 20
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
    }

    fn render_customize_badge_screen(&self) -> Element<Message> {
        let display_image_handle = self
            .selected_customize_image
            .as_ref()
            .map_or_else(|| BADGE_PLACEHOLDER_IMAGE.clone(), |h| h.clone());

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

        let images_to_select = [&DEFCON_LOGO_IMAGE, &DOGE_IMAGE, &PUPPY_IMAGE, &PIP_BOY_IMAGE, &VEGAS_IMAGE];

        let mut image_selection_row = row![]
            .spacing(10)
            .align_items(Alignment::Center)
            .width(Length::Fill);

        for img_handle_lazy in images_to_select.iter() {
            let img_handle_to_compare = (**img_handle_lazy).clone();
            let is_selected = self
                .selected_customize_image
                .as_ref()
                .map_or(false, |selected| selected.eq(&img_handle_to_compare));

            let button_style = if is_selected {
                theme_fn(SelectedBadgeStyle)
            } else {
                theme_fn(DefaultBadgeStyle)
            };

            let image_button_content: iced::widget::Image<image::Handle> =
                image((**img_handle_lazy).clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(ContentFit::ScaleDown);

            let image_button: iced::widget::Button<'_, Message, Theme, iced::Renderer> =
                button(image_button_content)
                    .on_press(Message::SelectCustomizeImage((**img_handle_lazy).clone()))
                    .padding(5)
                    .style(button_style);

            image_selection_row = image_selection_row.push(
                container(image_button)
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(160.0))
                    .center_x()
                    .center_y(),
            );
        }

        let back_button = button(text("Back").size(BUTTON_TEXT_SIZE))
            .on_press(Message::NavigateTo(AppScreen::Welcome))
            .padding([10, 40])
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

        let finish_button = button(text("Next").size(BUTTON_TEXT_SIZE))
            .on_press_maybe(if finish_button_enabled {
                Some(finish_button_message)
            } else {
                None
            })
            .padding([10, 40])
            .style(finish_button_style);

        // Create main content without buttons
        let main_content = column![
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
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill);

        // Create bottom navigation buttons positioned at bottom corners
        let bottom_navigation = container(
            row![back_button, Space::with_width(Length::Fill), finish_button,]
                .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(20);

        column![
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
            bottom_navigation
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn render_customize_leds_screen(&self) -> Element<Message> {
        let back_button = button(text("Back").size(BUTTON_TEXT_SIZE))
            .on_press(Message::NavigateTo(AppScreen::CustomizeBadge))
            .padding([10, 40])
            .style(theme_fn(YellowButtonStyle));

        let next_button = button(text("Next").size(BUTTON_TEXT_SIZE))
            .on_press(Message::NavigateTo(AppScreen::NameBadge))
            .padding([10, 40])
            .style(theme_fn(YellowButtonStyle));

        let modes = [
            LedMode::Manual,
            LedMode::Rainbow,
            LedMode::Snowstorm,
            LedMode::RedChase,
            LedMode::RainbowChase,
            LedMode::BlueChase,
            LedMode::GreenDot,
            LedMode::BlueDot,
            LedMode::BlueSin,
            LedMode::WhiteFade,
            LedMode::BarGraph,
            LedMode::Zylon,
            LedMode::Audio,
            LedMode::Accel,
        ];

        // Create two columns for better layout
        let radio_buttons = modes.chunks(7).enumerate().fold(
            row!().spacing(80).align_items(Alignment::Start),
            |row_acc, (_col_idx, chunk)| {
                let column = chunk.iter().fold(
                    column!().spacing(18).align_items(Alignment::Start),
                    |col_acc, mode| {
                        col_acc.push(
                            radio(
                                mode.display_name(),
                                *mode,
                                self.selected_led_mode,
                                Message::SelectLedMode,
                            )
                            .size(20)
                            .spacing(10),
                        )
                    },
                );
                row_acc.push(column)
            },
        );

        // Create main content without buttons
        let main_content = column![
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
            container(radio_buttons).width(Length::Fill).center_x(),
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill);

        // Create bottom navigation buttons positioned at bottom corners
        let bottom_navigation = container(
            row![back_button, Space::with_width(Length::Fill), next_button,]
                .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(20);

        column![
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
            bottom_navigation
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn render_name_badge_screen(&self) -> Element<Message> {
        let back_button = button(text("Back").size(BUTTON_TEXT_SIZE))
            .on_press(Message::NavigateTo(AppScreen::CustomizeLeds))
            .padding([10, 40])
            .style(theme_fn(YellowButtonStyle));

        let submit_button = button(text("Submit").size(BUTTON_TEXT_SIZE))
            .on_press(Message::NavigateTo(AppScreen::Summary))
            .padding([10, 40])
            .style(theme_fn(YellowButtonStyle));

        // Create the badge image container with consistent sizing
        let badge_image_container = container(
            image(NAME_ME_IMAGE.clone())
                .width(Length::Fixed(300.0))
                .height(Length::Fixed(300.0))
                .content_fit(ContentFit::ScaleDown),
        )
        .width(Length::Fixed(320.0))
        .height(Length::Fixed(320.0))
        .center_x()
        .center_y()
        .style(theme_fn_container(UserImageBorderStyle));

        // Create the input section with better spacing and centering
        let character_count = self.badge_name.len();
        let characters_remaining = 23 - character_count;
        let counter_color = if characters_remaining <= 3 {
            Color::from_rgb8(200, 0, 0) // Red when close to limit
        } else if characters_remaining <= 7 {
            Color::from_rgb8(255, 140, 0) // Orange when getting close
        } else {
            Color::from_rgb8(100, 100, 100) // Gray when plenty of room
        };

        let input_section = container(
            column![
                text("Type Your Badge Name Below")
                    .size(HEADING_SIZE)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .width(Length::Fill),
                text("(Letters and numbers only)")
                    .size(16)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .style(iced::theme::Text::Color(Color::from_rgb8(100, 100, 100)))
                    .width(Length::Fill),
                Space::new(Length::Shrink, Length::Fixed(20.0)),
                container(
                    text_input::<_, Theme, iced::Renderer>("Enter name...", &self.badge_name)
                        .on_input(Message::BadgeNameChanged)
                        .on_submit(Message::NavigateTo(AppScreen::Summary))
                        .padding(15)
                        .size(BODY_SIZE)
                        .width(Length::Fixed(300.0))
                        .id(text_input::Id::new(BADGE_NAME_INPUT_ID))
                )
                .width(Length::Fill)
                .center_x(),
                Space::new(Length::Shrink, Length::Fixed(10.0)),
                text(format!("{} characters remaining", characters_remaining))
                    .size(14)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .style(iced::theme::Text::Color(counter_color))
                    .width(Length::Fill),
            ]
            .align_items(Alignment::Center)
            .spacing(5),
        )
        .width(Length::Fixed(400.0))
        .center_x()
        .center_y();

        // Create main content without buttons
        let main_content = column![
            Space::with_height(Length::Fixed(30.0)),
            image(PLACE_ME_ICON.clone())
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0)),
            text("Place Me")
                .size(HEADING_SIZE)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::new(Length::Shrink, Length::Fixed(15.0)),
            // Main content area with better proportions
            container(
                row![
                    badge_image_container,
                    Space::with_width(Length::Fixed(40.0)),
                    input_section,
                ]
                .align_items(Alignment::Center)
                .width(Length::Shrink)
            )
            .width(Length::Fill)
            .center_x(),
        ]
        .spacing(15)
        .align_items(Alignment::Center)
        .width(Length::Fill);

        // Create bottom navigation buttons positioned at bottom corners
        let bottom_navigation = container(
            row![back_button, Space::with_width(Length::Fill), submit_button,]
                .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(20);

        column![
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
            bottom_navigation
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn render_summary_screen(&self) -> Element<Message> {
        let back_button = button(text("Back").size(BUTTON_TEXT_SIZE))
            .on_press(Message::NavigateTo(AppScreen::NameBadge))
            .padding([10, 40])
            .style(theme_fn(YellowButtonStyle));

        let configure_button_text = if self.is_configuring {
            "Configuring..."
        } else {
            "Configure Device"
        };

        let configure_button_enabled = !self.is_configuring;
        let configure_button_style = if configure_button_enabled {
            theme_fn(YellowButtonStyle)
        } else {
            theme_fn(DisabledButtonStyle)
        };

        let configure_button = button(text(configure_button_text).size(BUTTON_TEXT_SIZE))
            .on_press_maybe(if configure_button_enabled {
                Some(Message::StartConfiguration)
            } else {
                None
            })
            .padding([10, 30])
            .style(configure_button_style);

        let done_button = button(text("Done").size(BUTTON_TEXT_SIZE))
            .on_press(Message::NavigateTo(AppScreen::Welcome))
            .padding([10, 40])
            .style(theme_fn(YellowButtonStyle));

        // Summary content
        let selected_image_display = match &self.selected_customize_image {
            Some(handle) => container(
                image(handle.clone())
                    .width(Length::Fixed(120.0))
                    .height(Length::Fixed(120.0))
                    .content_fit(ContentFit::ScaleDown),
            )
            .width(Length::Fixed(140.0))
            .height(Length::Fixed(140.0))
            .center_x()
            .center_y()
            .style(theme_fn_container(UserImageBorderStyle)),
            None => container(
                text("No Image\nSelected")
                    .size(16)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .style(iced::theme::Text::Color(Color::from_rgb8(150, 150, 150))),
            )
            .width(Length::Fixed(140.0))
            .height(Length::Fixed(140.0))
            .center_x()
            .center_y()
            .style(theme_fn_container(UserImageBorderStyle)),
        };

        let selected_led_text = match &self.selected_led_mode {
            Some(mode) => mode.display_name(),
            None => "No LED Pattern Selected",
        };

        let badge_name_text = if self.badge_name.is_empty() {
            "No Name Entered"
        } else {
            &self.badge_name
        };

        // Create main content without buttons
        let main_content = column![
            Space::with_height(Length::Fixed(15.0)),
            text("Configuration Summary")
                .size(HEADING_SIZE)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::new(Length::Shrink, Length::Fixed(30.0)),
            container(
                column![
                    // Badge Image Section
                    column![
                        text("Badge Image:")
                            .size(BODY_SIZE + 2)
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                        Space::new(Length::Shrink, Length::Fixed(10.0)),
                        container(selected_image_display)
                            .width(Length::Fill)
                            .center_x(),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(5),
                    Space::new(Length::Shrink, Length::Fixed(25.0)),
                    // LED Pattern and Badge Name Section
                    column![
                        row![
                            text("LED Pattern: ").size(BODY_SIZE + 2),
                            text(selected_led_text)
                                .size(BODY_SIZE + 2)
                                .style(iced::theme::Text::Color(*BLUE_TEXT)),
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                        Space::new(Length::Shrink, Length::Fixed(15.0)),
                        row![
                            text("Badge Name: ").size(BODY_SIZE + 2),
                            text(badge_name_text)
                                .size(BODY_SIZE + 2)
                                .style(iced::theme::Text::Color(*BLUE_TEXT)),
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                    ]
                    .align_items(Alignment::Center)
                ]
                .align_items(Alignment::Center)
            )
            .padding(30)
            .style(theme_fn_container(SummaryBoxStyle)),
            Space::new(Length::Shrink, Length::Fixed(40.0)),
            // Configuration section
            column![
                text("Device Configuration")
                    .size(HEADING_SIZE - 5)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::new(Length::Shrink, Length::Fixed(20.0)),
                configure_button,
                Space::new(Length::Shrink, Length::Fixed(20.0)),
                if self.is_configuring || self.configuration_progress > 0.0 {
                    let status_text = if self.is_configuring {
                        &self.configuration_status
                    } else if self.configuration_error.is_some() {
                        "Configuration failed - see details below"
                    } else if self.configuration_progress >= 1.0 {
                        "Configuration Complete! You can configure again anytime."
                    } else {
                        &format!("{}%", (self.configuration_progress * 100.0) as u32)
                    };

                    let mut status_column = column![
                        progress_bar(0.0..=1.0, self.configuration_progress)
                            .width(Length::Fixed(400.0))
                            .height(Length::Fixed(20.0)),
                        Space::new(Length::Shrink, Length::Fixed(10.0)),
                        text(status_text)
                            .size(16)
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                    ];

                    // Add console output and error display side by side if there's any
                    let has_console = !self.configuration_console_output.is_empty();
                    let has_error = self.configuration_error.is_some();
                    
                    if has_console || has_error {
                        status_column = status_column.push(Space::new(Length::Shrink, Length::Fixed(15.0)));
                        
                        // Create a horizontal row for console and error
                        let mut console_row = row!().spacing(20).align_items(Alignment::Start);
                        
                        // Add console output if present
                        if has_console {
                            let console_column = column![
                                text("Console Output:")
                                    .size(14)
                                    .style(iced::theme::Text::Color(*BLUE_TEXT)),
                                Space::new(Length::Shrink, Length::Fixed(5.0)),
                                container(
                                    text(&self.configuration_console_output)
                                        .size(12)
                                        .style(iced::theme::Text::Color(Color::from_rgb(0.9, 0.9, 0.9)))
                                        .horizontal_alignment(iced::alignment::Horizontal::Left)
                                )
                                .width(Length::Fixed(if has_error { 480.0 } else { 800.0 }))
                                .height(Length::Fixed(250.0))
                                .padding(10)
                                .style(theme_fn_container(ConsoleOutputStyle))
                            ]
                            .spacing(5);
                            
                            console_row = console_row.push(console_column);
                        }
                        
                        // Add error message if present
                        if let Some(error) = &self.configuration_error {
                            let error_column = column![
                                text("Error:")
                                    .size(14)
                                    .style(iced::theme::Text::Color(Color::from_rgb8(200, 0, 0))),
                                Space::new(Length::Shrink, Length::Fixed(5.0)),
                                container(
                                    text(error)
                                        .size(14)
                                        .style(iced::theme::Text::Color(Color::from_rgb8(200, 0, 0)))
                                        .horizontal_alignment(iced::alignment::Horizontal::Left),
                                )
                                .width(Length::Fixed(if has_console { 480.0 } else { 800.0 }))
                                .height(Length::Fixed(250.0))
                                .padding(10)
                                .style(theme_fn_container(ErrorBoxStyle))
                            ]
                            .spacing(5);
                            
                            console_row = console_row.push(error_column);
                        }
                        
                        status_column = status_column.push(console_row);
                    }

                    container(status_column.align_items(Alignment::Center))
                        .width(Length::Fill)
                        .center_x()
                } else {
                    container(Space::new(Length::Shrink, Length::Fixed(50.0)))
                }
            ]
            .align_items(Alignment::Center)
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill);

        // Create bottom navigation buttons positioned at bottom corners
        let bottom_navigation = container(
            row![back_button, Space::with_width(Length::Fill), done_button,]
                .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(20);

        column![
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
            bottom_navigation
        ]
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
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let active = self.active(style);
        ButtonAppearance {
            background: Some(Color { a: 0.8, ..*YELLOW }.into()),
            ..active
        }
    }
}

struct DisabledButtonStyle;
impl ButtonStyleSheet for DisabledButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(Color::from_rgb(0.7, 0.7, 0.7).into()),
            text_color: Color::from_rgb(0.4, 0.4, 0.4),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

struct DefaultBadgeStyle;
impl ButtonStyleSheet for DefaultBadgeStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(Color::WHITE.into()),
            text_color: Color::BLACK,
            border: Border {
                color: Color::BLACK,
                width: 1.0,
                radius: 8.0.into(),
            },
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
            background: Some(Color { a: 0.3, ..*YELLOW }.into()),
            text_color: Color::BLACK,
            border: Border {
                color: *YELLOW,
                width: 2.0,
                radius: 8.0.into(),
            },
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

struct SummaryBoxStyle;
impl ContainerStyleSheet for SummaryBoxStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> ContainerAppearance {
        ContainerAppearance {
            background: Some(Color::from_rgb(0.95, 0.95, 0.95).into()),
            border: Border {
                color: Color::from_rgb(0.7, 0.7, 0.7),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

struct ErrorBoxStyle;
impl ContainerStyleSheet for ErrorBoxStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> ContainerAppearance {
        ContainerAppearance {
            background: Some(Color::from_rgb(1.0, 0.95, 0.95).into()),
            border: Border {
                color: Color::from_rgb8(200, 0, 0),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

struct ConsoleOutputStyle;
impl ContainerStyleSheet for ConsoleOutputStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> ContainerAppearance {
        ContainerAppearance {
            background: Some(Color::from_rgb(0.05, 0.05, 0.05).into()),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
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

fn configuration_subscription(
    selected_image: Option<image::Handle>,
    selected_led_mode: Option<LedMode>,
    badge_name: String,
) -> Subscription<Message> {
    iced::subscription::unfold(
        std::any::TypeId::of::<ConfigurationState>(),
        ConfigurationState::Start,
        move |state| {
            let selected_image = selected_image.clone();
            let selected_led_mode = selected_led_mode;
            let badge_name = badge_name.clone();
            
            async move {
                match state {
                    ConfigurationState::Start => {
                        // Step 1: Create configuration files
                        println!("Configuration: Starting configuration process");
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        
                        let config_content = create_config_content(selected_led_mode, badge_name.clone());
                        let config_file = "build_a_badge.txt";
                        println!("Configuration: Creating config file '{}' with content:\n{}", config_file, config_content);
                        if fs::write(config_file, &config_content).is_err() {
                            let error_msg = format!("Failed to write configuration file: {}", config_file);
                            println!("Configuration ERROR: {}", error_msg);
                            return (
                                Message::ConfigurationComplete(Err(error_msg)),
                                ConfigurationState::Done,
                            );
                        }
                        println!("Configuration: Successfully wrote config file");

                        let settings_content = create_settings_content(badge_name.clone());
                        let settings_file = "settings.txt";
                        println!("Configuration: Creating settings file '{}' with content:\n{}", settings_file, settings_content);
                        if fs::write(settings_file, settings_content).is_err() {
                            let error_msg = format!("Failed to write settings file: {}", settings_file);
                            println!("Configuration ERROR: {}", error_msg);
                            return (
                                Message::ConfigurationComplete(Err(error_msg)),
                                ConfigurationState::Done,
                            );
                        }
                        println!("Configuration: Successfully wrote settings file");

                        let step_message = "Step 1: Uploading configuration file...";
                        let detailed_message = format!("Generated configuration file content:\n{}\nStep 1: Uploading configuration file...", config_content);
                        println!("Configuration: {}", detailed_message);
                        (
                            Message::ConfigurationStepUpdate(step_message.to_string(), 0.1),
                            ConfigurationState::UploadConfig,
                        )
                    }
                    ConfigurationState::UploadConfig => {
                        // Step 1: Upload configuration file
                        println!("Configuration: Starting upload of configuration file");
                        
                        // Add timeout to prevent hanging
                        let result = tokio::time::timeout(
                            Duration::from_secs(30), // 10 second timeout
                            tokio::process::Command::new("fwi-serial")
                                .arg("-s")
                                .arg("build_a_badge.txt")
                                .arg("-fn")
                                .arg("/build_a_badge.txt")
                                .arg("-mi")
                                .arg("1")
                                .output()
                        ).await;

                        let (success, message) = match result {
                            Ok(Ok(output)) => {
                                let console_output = format!("Configuration: fwi-serial command completed with exit status: {}", output.status);
                                println!("{}", console_output);
                                
                                let mut full_output = console_output;
                                
                                if !output.stdout.is_empty() {
                                    let stdout_output = format!("Configuration: stdout: {}", String::from_utf8_lossy(&output.stdout));
                                    println!("{}", stdout_output);
                                    full_output.push_str(&format!("\n{}", stdout_output));
                                }
                                if !output.stderr.is_empty() {
                                    let stderr_output = format!("Configuration: stderr: {}", String::from_utf8_lossy(&output.stderr));
                                    println!("{}", stderr_output);
                                    full_output.push_str(&format!("\n{}", stderr_output));
                                }
                                
                                if output.status.success() {
                                    let success_msg = "✓ Configuration file uploaded successfully\nStep 2: Uploading image file...".to_string();
                                    let final_msg = format!("{}\nConfiguration: {}", full_output, success_msg);
                                    println!("Configuration: {}", success_msg);
                                    (true, final_msg)
                                } else {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    let error_msg = format!("✗ Configuration upload failed: {}\nConfiguration stopped due to error.", stderr);
                                    let final_msg = format!("{}\nConfiguration ERROR: {}", full_output, error_msg);
                                    println!("Configuration ERROR: {}", error_msg);
                                    (false, final_msg)
                                }
                            }
                            Ok(Err(e)) => {
                                let msg = format!("✗ Configuration upload error: {}\nConfiguration stopped due to error.", e);
                                println!("Configuration ERROR: {}", msg);
                                (false, msg)
                            },
                            Err(_) => {
                                let msg = "✗ Configuration upload timed out (10 seconds) - device may not be connected\nConfiguration stopped due to error.".to_string();
                                println!("Configuration ERROR: {}", msg);
                                println!("Configuration: Timeout occurred, returning error message");
                                (false, msg)
                            }
                        };

                        if !success {
                            println!("Configuration: Returning error ConfigurationComplete with message: {}", message);
                            return (
                                Message::ConfigurationComplete(Err(message.clone())),
                                ConfigurationState::Done,
                            );
                        }

                        (
                            Message::ConfigurationStepUpdate("Step 2: Uploading image file...".to_string(), 0.3),
                            ConfigurationState::UploadImage,
                        )
                    }
                    ConfigurationState::UploadImage => {
                        // Step 2: Upload image file
                        println!("Configuration: Starting upload of image file");
                        
                        // Determine the correct image file path based on selection
                        let image_file_path = match selected_image.as_ref() {
                            Some(handle) => {
                                // Map the image handle to the corresponding .fwi file
                                if handle == &*DEFCON_LOGO_IMAGE {
                                    "assets/defcon_logo.fwi"
                                } else if handle == &*DOGE_IMAGE {
                                    "assets/doge.fwi"
                                } else if handle == &*PUPPY_IMAGE {
                                    "assets/puppy.fwi"
                                } else if handle == &*PIP_BOY_IMAGE {
                                    "assets/pip_boy.fwi"
                                } else if handle == &*VEGAS_IMAGE {
                                    "assets/vegas.fwi"
                                } else {
                                    "assets/badge_placeholder.fwi"
                                }
                            }
                            None => "assets/badge_placeholder.fwi"
                        };
                        
                        println!("Configuration: Using image file: {}", image_file_path);
                        
                        let result = tokio::time::timeout(
                            Duration::from_secs(30), // 10 second timeout
                            tokio::process::Command::new("fwi-serial")
                                .arg("-s")
                                .arg(image_file_path)
                                .arg("-fn")
                                .arg("/images/build_a_badge.fwi")
                                .output()
                        ).await;

                        let (success, message) = match result {
                            Ok(Ok(output)) => {
                                let console_output = format!("Configuration: fwi-serial image upload completed with exit status: {}", output.status);
                                println!("{}", console_output);
                                
                                let mut full_output = console_output;
                                
                                if !output.stdout.is_empty() {
                                    let stdout_output = format!("Configuration: stdout: {}", String::from_utf8_lossy(&output.stdout));
                                    println!("{}", stdout_output);
                                    full_output.push_str(&format!("\n{}", stdout_output));
                                }
                                if !output.stderr.is_empty() {
                                    let stderr_output = format!("Configuration: stderr: {}", String::from_utf8_lossy(&output.stderr));
                                    println!("{}", stderr_output);
                                    full_output.push_str(&format!("\n{}", stderr_output));
                                }
                                
                                if output.status.success() {
                                    let success_msg = "✓ Image file uploaded successfully\nStep 3: Uploading WASM file...".to_string();
                                    let final_msg = format!("{}\nConfiguration: {}", full_output, success_msg);
                                    println!("Configuration: {}", success_msg);
                                    (true, final_msg)
                                } else {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    let error_msg = format!("✗ Image upload failed: {}\nConfiguration stopped due to error.", stderr);
                                    let final_msg = format!("{}\nConfiguration ERROR: {}", full_output, error_msg);
                                    println!("Configuration ERROR: {}", error_msg);
                                    (false, final_msg)
                                }
                            }
                            Ok(Err(e)) => {
                                let msg = format!("✗ Image upload error: {}\nConfiguration stopped due to error.", e);
                                println!("Configuration ERROR: {}", msg);
                                (false, msg)
                            },
                            Err(_) => {
                                let msg = "✗ Image upload timed out (10 seconds) - device may not be connected\nConfiguration stopped due to error.".to_string();
                                println!("Configuration ERROR: {}", msg);
                                (false, msg)
                            }
                        };

                        if !success {
                            return (
                                Message::ConfigurationComplete(Err(message.clone())),
                                ConfigurationState::Done,
                            );
                        }

                        (
                            Message::ConfigurationStepUpdate("Step 3: Uploading WASM file...".to_string(), 0.5),
                            ConfigurationState::UploadWasm,
                        )
                    }
                    ConfigurationState::UploadWasm => {
                        // Step 3: Upload WASM file (allowed to fail)
                        println!("Configuration: Starting upload of WASM file (expected to fail if file doesn't exist)");
                        
                        let result = tokio::time::timeout(
                            Duration::from_secs(30), // 10 second timeout
                            tokio::process::Command::new("fwi-serial")
                                .arg("-s")
                                .arg("build_a_badge.wasm")
                                .output()
                        ).await;

                        let _message = match result {
                            Ok(Ok(output)) => {
                                let console_output = format!("Configuration: fwi-serial WASM upload completed with exit status: {}", output.status);
                                println!("{}", console_output);
                                
                                let mut full_output = console_output;
                                
                                if !output.stdout.is_empty() {
                                    let stdout_output = format!("Configuration: stdout: {}", String::from_utf8_lossy(&output.stdout));
                                    println!("{}", stdout_output);
                                    full_output.push_str(&format!("\n{}", stdout_output));
                                }
                                if !output.stderr.is_empty() {
                                    let stderr_output = format!("Configuration: stderr: {}", String::from_utf8_lossy(&output.stderr));
                                    println!("{}", stderr_output);
                                    full_output.push_str(&format!("\n{}", stderr_output));
                                }
                                
                                if output.status.success() {
                                    let msg = "✓ WASM file uploaded successfully\nStep 4: Uploading settings file...".to_string();
                                    let final_msg = format!("{}\nConfiguration: {}", full_output, msg);
                                    println!("Configuration: {}", msg);
                                    final_msg
                                } else {
                                    let msg = "✗ WASM upload failed (expected - file doesn't exist yet)\nStep 4: Uploading settings file...".to_string();
                                    let final_msg = format!("{}\nConfiguration: {}", full_output, msg);
                                    println!("Configuration: {}", msg);
                                    final_msg
                                }
                            }
                            Ok(Err(e)) => {
                                let msg = "✗ WASM upload error (expected - file doesn't exist yet)\nStep 4: Uploading settings file...".to_string();
                                println!("Configuration: {} (Error: {})", msg, e);
                                msg
                            },
                            Err(_) => {
                                let msg = "✗ WASM upload timed out (expected - file doesn't exist yet)\nStep 4: Uploading settings file...".to_string();
                                println!("Configuration: {}", msg);
                                msg
                            }
                        };

                        // WASM failure is expected and doesn't stop the process
                        (
                            Message::ConfigurationStepUpdate("Step 4: Uploading settings file...".to_string(), 0.7),
                            ConfigurationState::UploadSettings,
                        )
                    }
                    ConfigurationState::UploadSettings => {
                        // Step 4: Upload settings file
                        println!("Configuration: Starting upload of settings file");
                        
                        let result = tokio::time::timeout(
                            Duration::from_secs(30), // 10 second timeout
                            tokio::process::Command::new("fwi-serial")
                                .arg("-s")
                                .arg("settings.txt")
                                .arg("-fn")
                                .arg("/settings.txt")
                                .arg("-mi")
                                .arg("1")
                                .output()
                        ).await;

                        let (success, final_message) = match result {
                            Ok(Ok(output)) => {
                                let console_output = format!("Configuration: fwi-serial settings upload completed with exit status: {}", output.status);
                                println!("{}", console_output);
                                
                                let mut full_output = console_output;
                                
                                if !output.stdout.is_empty() {
                                    let stdout_output = format!("Configuration: stdout: {}", String::from_utf8_lossy(&output.stdout));
                                    println!("{}", stdout_output);
                                    full_output.push_str(&format!("\n{}", stdout_output));
                                }
                                if !output.stderr.is_empty() {
                                    let stderr_output = format!("Configuration: stderr: {}", String::from_utf8_lossy(&output.stderr));
                                    println!("{}", stderr_output);
                                    full_output.push_str(&format!("\n{}", stderr_output));
                                }
                                
                                if output.status.success() {
                                    let success_msg = "✓ Settings file uploaded successfully".to_string();
                                    let final_msg = format!("{}\nConfiguration: {}", full_output, success_msg);
                                    println!("Configuration: {}", success_msg);
                                    (true, final_msg)
                                } else {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    let error_msg = format!("✗ Settings upload failed: {}\nConfiguration stopped due to error.", stderr);
                                    let final_msg = format!("{}\nConfiguration ERROR: {}", full_output, error_msg);
                                    println!("Configuration ERROR: {}", error_msg);
                                    (false, final_msg)
                                }
                            }
                            Ok(Err(e)) => {
                                let msg = format!("✗ Settings upload error: {}\nConfiguration stopped due to error.", e);
                                println!("Configuration ERROR: {}", msg);
                                (false, msg)
                            },
                            Err(_) => {
                                let msg = "✗ Settings upload timed out (10 seconds) - device may not be connected\nConfiguration stopped due to error.".to_string();
                                println!("Configuration ERROR: {}", msg);
                                (false, msg)
                            }
                        };

                        if !success {
                            return (
                                Message::ConfigurationComplete(Err(final_message)),
                                ConfigurationState::Done,
                            );
                        }

                        (
                            Message::ConfigurationStepUpdate("Step 5: Running WASM application...".to_string(), 0.9),
                            ConfigurationState::RunWasm,
                        )
                    }
                    ConfigurationState::RunWasm => {
                        // Step 5: Run WASM application
                        println!("Configuration: Starting WASM application execution");
                        
                        let result = tokio::time::timeout(
                            Duration::from_secs(30),
                            tokio::process::Command::new("fwi-serial")
                                .arg("-w")
                                .arg("build_a_badge.wasm")
                                .output()
                        ).await;

                        let (success, final_message) = match result {
                            Ok(Ok(output)) => {
                                let console_output = format!("Configuration: fwi-serial WASM run completed with exit status: {}", output.status);
                                println!("{}", console_output);
                                
                                let mut full_output = console_output;
                                
                                if !output.stdout.is_empty() {
                                    let stdout_output = format!("Configuration: stdout: {}", String::from_utf8_lossy(&output.stdout));
                                    println!("{}", stdout_output);
                                    full_output.push_str(&format!("\n{}", stdout_output));
                                }
                                if !output.stderr.is_empty() {
                                    let stderr_output = format!("Configuration: stderr: {}", String::from_utf8_lossy(&output.stderr));
                                    println!("{}", stderr_output);
                                    full_output.push_str(&format!("\n{}", stderr_output));
                                }
                                
                                if output.status.success() {
                                    let success_msg = "✓ WASM application executed successfully".to_string();
                                    let final_msg = format!("{}\nConfiguration: {}", full_output, success_msg);
                                    println!("Configuration: {}", success_msg);
                                    (true, final_msg)
                                } else {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    let error_msg = format!("✗ WASM execution failed: {}\nConfiguration stopped due to error.", stderr);
                                    let final_msg = format!("{}\nConfiguration ERROR: {}", full_output, error_msg);
                                    println!("Configuration ERROR: {}", error_msg);
                                    (false, final_msg)
                                }
                            }
                            Ok(Err(e)) => {
                                let msg = format!("✗ WASM execution error: {}\nConfiguration stopped due to error.", e);
                                println!("Configuration ERROR: {}", msg);
                                (false, msg)
                            },
                            Err(_) => {
                                let msg = "✗ WASM execution timed out (30 seconds) - device may not be connected\nConfiguration stopped due to error.".to_string();
                                println!("Configuration ERROR: {}", msg);
                                (false, msg)
                            }
                        };

                        let result = if success {
                            println!("Configuration: All steps completed successfully!");
                            Ok("Configuration completed successfully!".to_string())
                        } else {
                            println!("Configuration: Process failed during WASM execution");
                            Err(final_message)
                        };

                        (
                            Message::ConfigurationComplete(result),
                            ConfigurationState::Done,
                        )
                    }
                    ConfigurationState::Done => {
                        // Configuration finished, wait indefinitely
                        println!("Configuration: Reached Done state - configuration process complete");
                        futures::future::pending().await
                    }
                }
            }
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConfigurationState {
    Start,
    UploadConfig,
    UploadImage,
    UploadWasm,
    UploadSettings,
    RunWasm,
    Done,
}

fn create_config_content(selected_led_mode: Option<LedMode>, badge_name: String) -> String {
    let led_pattern = match selected_led_mode {
        Some(mode) => mode.as_integer().to_string(),
        None => "0".to_string(), // Default to Manual (0)
    };

    let name = if badge_name.is_empty() {
        "Boring"
    } else {
        &badge_name
    };

    format!("{name}-WiLi\n{led_pattern}\n")
}

fn create_settings_content(badge_name: String) -> String {
    format!(
        "wifiAPEn=1
wifiAPssid={badge_name}-WiLi
wifiAPAuth=0
btEn=1
btAPen={badge_name}-WiLi
btTerm=1\n"
    )
}
