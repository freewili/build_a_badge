// main.rs
use iced::widget::{button, column, container, row, text, Space, image, radio, text_input, progress_bar};
use iced::{
    Alignment, Element, Length, Settings, Theme, Color, Size, Border,
    Application, Command, Subscription, executor,
    ContentFit,
};
use iced::window;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use std::fs;

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
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LedMode {
    Off,
    Rainbow,
    Snowstorm,
    RedChase,
    RainbowChase,
    BlueChase,
    GreenDot,
    BlueSin,
    WhiteFade,
    Accelerometer,
}

impl LedMode {
    fn display_name(&self) -> &'static str {
        match self {
            LedMode::Off => "Off",
            LedMode::Rainbow => "Rainbow",
            LedMode::Snowstorm => "Snowstorm",
            LedMode::RedChase => "Red Chase",
            LedMode::RainbowChase => "Rainbow Chase",
            LedMode::BlueChase => "Blue Chase",
            LedMode::GreenDot => "Green Dot",
            LedMode::BlueSin => "Blue Sin",
            LedMode::WhiteFade => "White Fade",
            LedMode::Accelerometer => "Accelerometer",
        }
    }
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
    
    // Configuration state
    is_configuring: bool,
    configuration_progress: f32,
    configuration_status: String,
    configuration_error: Option<String>,
    
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
    StartConfiguration,
    ConfigurationProgress,
    ConfigurationComplete(Result<String, String>),
    
    // Animation messages
    Tick(Instant),
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
            selected_led_mode: None,
            badge_name: String::new(),
            
            is_configuring: false,
            configuration_progress: 0.0,
            configuration_status: String::new(),
            configuration_error: None,
            
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
                    
                    // Clear configuration status when navigating away from summary
                    if self.current_screen == AppScreen::Summary {
                        self.configuration_status = String::new();
                        self.configuration_error = None;
                        self.configuration_progress = 0.0;
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
                let filtered_name: String = name.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect();
                
                if filtered_name.len() <= 23 {
                    self.badge_name = filtered_name;
                }
            }
            Message::StartConfiguration => {
                self.is_configuring = true;
                self.configuration_progress = 0.0;
                self.configuration_status = "Starting configuration...".to_string();
                self.configuration_error = None;
                
                // Create configuration and upload it
                return Command::perform(
                    configure_device(
                        self.selected_customize_image.clone(),
                        self.selected_led_mode,
                        self.badge_name.clone()
                    ),
                    Message::ConfigurationComplete
                );
            }
            Message::ConfigurationProgress => {
                if self.is_configuring && self.configuration_progress < 1.0 {
                    self.configuration_progress += 0.02; // Increment by 2% each update
                    if self.configuration_progress >= 1.0 {
                        self.configuration_progress = 1.0;
                        self.is_configuring = false;
                    }
                }
            }
            Message::ConfigurationComplete(result) => {
                self.is_configuring = false;
                self.configuration_progress = 1.0;
                
                match result {
                    Ok(message) => {
                        println!("Configuration successful: {}", message);
                        self.configuration_status = "Configuration successful!".to_string();
                        self.configuration_error = None;
                    }
                    Err(error) => {
                        println!("Configuration failed: {}", error);
                        self.configuration_status = "Configuration failed".to_string();
                        self.configuration_error = Some(error);
                    }
                }
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
                
                // Update configuration progress if configuring
                if self.is_configuring {
                    return self.update(Message::ConfigurationProgress);
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
            AppScreen::Summary => self.render_summary_screen(),
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

    // Create a container for the logo with a smaller, responsive height
    let app_logo_container = container(
        image(APP_LOGO_IMAGE.clone())
            .width(Length::Fixed(300.0)) // Reduced from 400 to 300
            .height(Length::Fixed(250.0)) // Reduced from 400 to 250
            .content_fit(ContentFit::ScaleDown) // Ensure the image scales down
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
            .on_press_maybe(if finish_button_enabled { Some(finish_button_message) } else { None })
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
            row![
                back_button,
                Space::with_width(Length::Fill),
                finish_button,
            ]
            .align_items(Alignment::Center)
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
            LedMode::Off,
            LedMode::Rainbow, 
            LedMode::Snowstorm,
            LedMode::RedChase,
            LedMode::RainbowChase,
            LedMode::BlueChase, 
            LedMode::GreenDot,
            LedMode::BlueSin,
            LedMode::WhiteFade,
            LedMode::Accelerometer
        ];

        // Create two columns for better layout
        let radio_buttons = modes.chunks(5).enumerate().fold(
            row!().spacing(80).align_items(Alignment::Start),
            |row_acc, (_col_idx, chunk)| {
                let column = chunk.iter().fold(
                    column!().spacing(18).align_items(Alignment::Start),
                    |col_acc, mode| {
                        col_acc.push(radio(
                            mode.display_name(),
                            *mode,
                            self.selected_led_mode,
                            Message::SelectLedMode,
                        )
                        .size(20)
                        .spacing(10))
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
            container(radio_buttons)
                .width(Length::Fill)
                .center_x(),
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill);

        // Create bottom navigation buttons positioned at bottom corners
        let bottom_navigation = container(
            row![
                back_button,
                Space::with_width(Length::Fill),
                next_button,
            ]
            .align_items(Alignment::Center)
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
                .content_fit(ContentFit::ScaleDown)
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
                        .padding(15)
                        .size(BODY_SIZE)
                        .width(Length::Fixed(300.0))
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
            .spacing(5)
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
            row![
                back_button,
                Space::with_width(Length::Fill),
                submit_button,
            ]
            .align_items(Alignment::Center)
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
            .on_press_maybe(if configure_button_enabled { Some(Message::StartConfiguration) } else { None })
            .padding([10, 30])
            .style(configure_button_style);

        let done_button = button(text("Done").size(BUTTON_TEXT_SIZE))
            .on_press(Message::NavigateTo(AppScreen::Welcome))
            .padding([10, 40])
            .style(theme_fn(YellowButtonStyle));

        // Summary content
        let selected_image_display = match &self.selected_customize_image {
            Some(handle) => {
                container(
                    image(handle.clone())
                        .width(Length::Fixed(120.0))
                        .height(Length::Fixed(120.0))
                        .content_fit(ContentFit::ScaleDown)
                )
                .width(Length::Fixed(140.0))
                .height(Length::Fixed(140.0))
                .center_x()
                .center_y()
                .style(theme_fn_container(UserImageBorderStyle))
            },
            None => {
                container(
                    text("No Image\nSelected")
                        .size(16)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                        .style(iced::theme::Text::Color(Color::from_rgb8(150, 150, 150)))
                )
                .width(Length::Fixed(140.0))
                .height(Length::Fixed(140.0))
                .center_x()
                .center_y()
                .style(theme_fn_container(UserImageBorderStyle))
            }
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
                            text(selected_led_text).size(BODY_SIZE + 2).style(iced::theme::Text::Color(*BLUE_TEXT)),
                        ].spacing(10).align_items(Alignment::Center),
                        Space::new(Length::Shrink, Length::Fixed(15.0)),
                        row![
                            text("Badge Name: ").size(BODY_SIZE + 2),
                            text(badge_name_text).size(BODY_SIZE + 2).style(iced::theme::Text::Color(*BLUE_TEXT)),
                        ].spacing(10).align_items(Alignment::Center),
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
                    
                    // Add error message if there's an error
                    if let Some(error) = &self.configuration_error {
                        status_column = status_column.push(Space::new(Length::Shrink, Length::Fixed(10.0)));
                        status_column = status_column.push(
                            container(
                                text(error)
                                    .size(14)
                                    .style(iced::theme::Text::Color(Color::from_rgb8(200, 0, 0)))
                                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                            )
                            .width(Length::Fixed(400.0))
                            .padding(10)
                            .style(theme_fn_container(ErrorBoxStyle))
                        );
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
            row![
                back_button,
                Space::with_width(Length::Fill),
                done_button,
            ]
            .align_items(Alignment::Center)
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

// Configuration function for device setup
async fn configure_device(
    selected_image: Option<image::Handle>,
    selected_led_mode: Option<LedMode>,
    badge_name: String,
) -> Result<String, String> {
    // Create configuration content
    let config_content = create_config_content(selected_image, selected_led_mode, badge_name);
    
    // Write configuration to file
    let config_file = "badge_config.fwi";
    match fs::write(config_file, config_content) {
        Ok(_) => {
            // Upload the file using fwi-serial
            match upload_config_file(config_file).await {
                Ok(output) => Ok(format!("Configuration uploaded successfully: {}", output)),
                Err(e) => Err(format!("Failed to upload configuration: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to write configuration file: {}", e)),
    }
}

fn create_config_content(
    selected_image: Option<image::Handle>,
    selected_led_mode: Option<LedMode>,
    badge_name: String,
) -> String {
    let image_name = match selected_image {
        Some(_) => "selected_image.png", // In a real implementation, you'd map the handle to actual image name
        None => "default.png",
    };
    
    let led_pattern = match selected_led_mode {
        Some(mode) => mode.display_name(),
        None => "Off",
    };
    
    let name = if badge_name.is_empty() {
        "Badge"
    } else {
        &badge_name
    };
    
    format!(
        r#"# Badge Configuration File
[DISPLAY]
image={image_name}
name={name}

[LED]
pattern={led_pattern}

[SETTINGS]
version=1.0
timestamp={}
"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
}

async fn upload_config_file(config_file: &str) -> Result<String, String> {
    // First, let's read and display the configuration file content for debugging
    match fs::read_to_string(config_file) {
        Ok(content) => {
            println!("Generated configuration file content:");
            println!("{}", content);
        }
        Err(_) => {
            println!("Could not read configuration file for preview");
        }
    }
    
    // Execute fwi-serial command
    let output = tokio::process::Command::new("fwi-serial")
        .arg("-s")
        .arg(config_file)
        .arg("-fn")
        .arg("/images/badge.fwi")
        .output()
        .await;
    
    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(String::from_utf8_lossy(&result.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                // For development, let's show what the command tried to do
                println!("fwi-serial command executed: fwi-serial -s {} -fn /images/badge.fwi", config_file);
                Err(stderr.to_string())
            }
        }
        Err(e) => Err(format!("Failed to execute fwi-serial: {}", e)),
    }
}