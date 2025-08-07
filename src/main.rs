// main.rs
use iced::widget::{button, column, container, row, text, Space, image, radio, text_input};
use iced::{
    Alignment, Element, Length, Settings, Theme, Color, Size, Border, Font,
    Application, Command, Subscription, executor,
    ContentFit,
};
use iced::window;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::pin::Pin;

// Audio specific imports
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use hound::{WavWriter, WavSpec};

// Explicitly import necessary types and traits for Iced 0.12.1
use iced::widget::button::{Appearance as ButtonAppearance, StyleSheet as ButtonStyleSheet};
use iced::widget::container::{Appearance as ContainerAppearance, StyleSheet as ContainerStyleSheet};

// Import Recipe from iced_futures::subscription
use iced_futures::subscription::Recipe;
// --- Constants and Statics ---
static YELLOW: LazyLock<Color> = LazyLock::new(|| Color::from_rgb8(255, 191, 0));
static GREEN: LazyLock<Color> = LazyLock::new(|| Color::from_rgb8(0, 150, 0));
static BLUE_TEXT: LazyLock<Color> = LazyLock::new(|| Color::from_rgb8(0, 85, 150));

// --- Static handles for your images ---
static BADGE_PLACEHOLDER_IMAGE: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/badge_placeholder.png").to_vec())
});
static STUFF_ME_ICON: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/stuff_me_icon.png").to_vec())
});
static HEAR_ME_ICON: LazyLock<image::Handle> = LazyLock::new(|| {
    image::Handle::from_memory(include_bytes!("../assets/hear_me_icon.png").to_vec())
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
const MONOSPACE_FONT: Font = Font::MONOSPACE;

// Animation constants
const TRANSITION_DURATION_MILLIS: u64 = 300; // Total duration for fade in/out
const ANIMATION_TICK_MILLIS: u64 = 16; // Roughly 60 FPS

// Audio recording constants
const MAX_RECORDING_SECONDS: f32 = 10.0;
const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 1; // Mono recording
const BITS_PER_SAMPLE: u16 = 16; // Signed 16-bit PCM

// NEW: Global sender for Audio Commands (for `update` to send to the Subscription's task)
static AUDIO_CMD_SENDER_GLOBAL: LazyLock<Arc<Mutex<Option<tokio::sync::mpsc::Sender<AudioCommand>>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

// --- Application State and Messages ---
#[derive(Debug, Clone, Copy, PartialEq)]
enum AppScreen {
    Welcome,
    CustomizeBadge,
    RecordSound,
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

// New enum for recording state
#[derive(Debug, Clone)]
enum RecordingState {
    Idle,
    Recording(Instant, PathBuf), // Store start time and path
    Stopped(PathBuf),
    Error(String),
}

// Messages specifically for the audio background task (must be Send)
#[derive(Debug)]
enum AudioCommand {
    Start(StreamConfig, SampleFormat, cpal::ChannelCount, Arc<Mutex<Vec<i16>>>),
    Stop,
}

// Messages from the audio background task back to the UI (must be Send + Clone)
#[derive(Debug, Clone)]
enum AudioEvent {
    StreamStarted,
    StreamError(String),
    RecordingDataReady(PathBuf),
    RecordingError(String),
}


struct BuildABadgeApp {
    current_screen: AppScreen,
    selected_top_banner_index: Option<usize>,
    selected_customize_image: Option<image::Handle>,
    selected_led_mode: Option<LedMode>,
    badge_name: String,
    
    // Animation state
    transition: AppScreenTransition,
    current_opacity: f32,
    transition_start_time: Instant,

    // Audio recording state
    recording_state: RecordingState,
    audio_buffer: Arc<Mutex<Vec<i16>>>, // Buffer to store captured samples
    timer_countdown: f32, // For displaying the countdown

    // Channels for communicating with the audio task
    // REMOVED: audio_cmd_sender: tokio::sync::mpsc::Sender<AudioCommand>, // This will be global now
    // The receiver will be handled directly in `new()` for the Subscription
}

#[derive(Debug, Clone)]
enum Message {
    NavigateTo(AppScreen),
    SelectCustomizeImage(image::Handle),
    SelectLedMode(LedMode),
    BadgeNameChanged(String),
    
    // Audio recording messages (these are now primary triggers for audio_cmd_sender)
    StartRecording,
    StopRecording,
    // Message from audio task back to UI
    AudioOutput(AudioEvent),
    
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
        let audio_buffer_app_ref = Arc::new(Mutex::new(Vec::new())); // Still needed by app logic

        // The initial application state
        let app_state = Self {
            current_screen: AppScreen::Welcome,
            selected_top_banner_index: Some(0),
            selected_customize_image: None,
            selected_led_mode: None,
            badge_name: String::new(),
            
            transition: AppScreenTransition::FadingIn,
            current_opacity: 0.0,
            transition_start_time: Instant::now(),

            recording_state: RecordingState::Idle,
            audio_buffer: audio_buffer_app_ref,
            timer_countdown: MAX_RECORDING_SECONDS,
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
                    // If leaving RecordSound screen, ensure recording is stopped/reset
                    if self.current_screen == AppScreen::RecordSound {
                        if let RecordingState::Recording(_, _) = self.recording_state {
                            println!("Navigating away, sending Stop command to audio task.");
                            // Send to global sender
                            if let Some(sender) = AUDIO_CMD_SENDER_GLOBAL.lock().unwrap().as_ref() {
                                let _ = sender.try_send(AudioCommand::Stop); // Non-blocking send
                            }
                            self.audio_buffer.lock().unwrap().clear();
                            self.recording_state = RecordingState::Idle;
                            self.timer_countdown = MAX_RECORDING_SECONDS;
                        }
                    }

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
            // --- Audio Recording Handlers (now send commands to background task) ---
            Message::StartRecording => {
                if matches!(self.recording_state, RecordingState::Idle | RecordingState::Stopped(_)) {
                    println!("UI: Attempting to start recording...");
                    self.audio_buffer.lock().unwrap().clear();
                    
                    let record_start_time = Instant::now();
                    let temp_wav_path = PathBuf::from("my_badge_sound.wav");
                    self.recording_state = RecordingState::Recording(record_start_time, temp_wav_path.clone());
                    self.timer_countdown = MAX_RECORDING_SECONDS;
                    
                    // We need to determine the config and format here to send to the audio task
                    let host = cpal::default_host();
                    let device_option = host.default_input_device();
                    
                    let (config, actual_sample_format, actual_channels) = 
                        match get_audio_config_and_format(device_option) {
                            Ok(res) => res,
                            Err(e) => {
                                self.recording_state = RecordingState::Error(e.clone());
                                eprintln!("UI: Failed to get audio config for starting: {}", e);
                                return Command::none();
                            }
                        };
                    
                    // Send Start command to global audio task sender
                    if let Some(sender) = AUDIO_CMD_SENDER_GLOBAL.lock().unwrap().as_ref() {
                        let _ = sender.try_send(AudioCommand::Start(
                            config, actual_sample_format, actual_channels, Arc::clone(&self.audio_buffer)
                        ));
                    }
                }
            }
            Message::StopRecording => {
                if let RecordingState::Recording(_, _temp_path) = &self.recording_state {
                    println!("UI: Sending Stop command to audio task.");
                    if let Some(sender) = AUDIO_CMD_SENDER_GLOBAL.lock().unwrap().as_ref() {
                        let _ = sender.try_send(AudioCommand::Stop);
                    }
                    
                    self.recording_state = RecordingState::Idle; 
                    self.timer_countdown = MAX_RECORDING_SECONDS;
                }
            }
            Message::AudioOutput(event) => {
                match event {
                    AudioEvent::StreamStarted => {
                        println!("UI: Audio stream reported started.");
                    }
                    AudioEvent::StreamError(e) => {
                        self.recording_state = RecordingState::Error(format!("Stream error: {}", e));
                        eprintln!("UI: Audio stream error: {}", e);
                    }
                    AudioEvent::RecordingDataReady(path) => {
                        self.recording_state = RecordingState::Stopped(path.clone());
                        println!("UI: Recording saved to: {:?}", path);
                    }
                    AudioEvent::RecordingError(e) => {
                        self.recording_state = RecordingState::Error(format!("Saving error: {}", e));
                        eprintln!("UI: Recording saving error: {}", e);
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

                if let AppScreen::RecordSound = self.current_screen {
                    if let RecordingState::Recording(start_time, _) = self.recording_state {
                        let elapsed_recording = (now - start_time).as_secs_f32();
                        self.timer_countdown = (MAX_RECORDING_SECONDS - elapsed_recording).max(0.0);

                        if self.timer_countdown <= 0.0 {
                            println!("UI: 10-second recording limit reached. Auto-stopping.");
                            if let Some(sender) = AUDIO_CMD_SENDER_GLOBAL.lock().unwrap().as_ref() {
                                let _ = sender.try_send(AudioCommand::Stop);
                            }
                        }
                    } else {
                        self.timer_countdown = MAX_RECORDING_SECONDS;
                    }
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            iced::time::every(Duration::from_millis(ANIMATION_TICK_MILLIS))
                .map(Message::Tick),
            iced::Subscription::from_recipe(AudioEventSubscription::new()),
        ])
    }

    fn view(&self) -> Element<Message> {
        let current_screen_element = match self.current_screen {
            AppScreen::Welcome => self.render_welcome_screen(),
            AppScreen::CustomizeBadge => self.render_customize_badge_screen(),
            AppScreen::RecordSound => self.render_record_sound_screen(),
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
            Message::NavigateTo(AppScreen::RecordSound)
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

    fn render_record_sound_screen(&self) -> Element<Message> {
        let back_button = button(text("Back").size(BODY_SIZE))
            .on_press(Message::NavigateTo(AppScreen::CustomizeBadge))
            .padding([5,20])
            .style(theme_fn(YellowButtonStyle));

        type RecordButton<'a> = Element<'a, Message, Theme, iced::Renderer>;

        let (record_button, stop_button, status_text_element, timer_text_element): (RecordButton, RecordButton, Element<Message>, Element<Message>) = match &self.recording_state {
            RecordingState::Idle | RecordingState::Stopped(_) | RecordingState::Error(_) => {
                let start_btn: RecordButton = button(text("Record").size(BODY_SIZE))
                    .on_press(Message::StartRecording)
                    .padding(10)
                    .style(theme_fn(YellowButtonStyle))
                    .into();
                let stop_btn: RecordButton = button(text("Stop").size(BODY_SIZE))
                    .style(theme_fn(DisabledButtonStyle))
                    .padding(10)
                    .into();
                
                let status_msg = match &self.recording_state {
                    RecordingState::Idle => "Press Record to start.".to_string(),
                    RecordingState::Stopped(path) => format!("Recording saved to: {}", path.display()),
                    RecordingState::Error(e) => format!("Error: {}", e),
                    _ => "".to_string(),
                };
                let status_color = match &self.recording_state {
                    RecordingState::Stopped(_) => *GREEN,
                    RecordingState::Error(_) => Color::from_rgb8(200,0,0),
                    _ => Color::BLACK,
                };

                (
                    start_btn,
                    stop_btn,
                    text(status_msg).size(BODY_SIZE).style(iced::theme::Text::Color(status_color)).into(),
                    text(format!("{:.1}s", MAX_RECORDING_SECONDS)).size(HEADING_SIZE).style(iced::theme::Text::Color(*BLUE_TEXT)).font(MONOSPACE_FONT).into(),
                )
            },
            RecordingState::Recording(_, _) => {
                let start_btn: RecordButton = button(text("Record").size(BODY_SIZE))
                    .style(theme_fn(DisabledButtonStyle))
                    .padding(10)
                    .into();
                let stop_btn: RecordButton = button(text("Stop").size(BODY_SIZE))
                    .on_press(Message::StopRecording)
                    .padding(10)
                    .style(theme_fn(YellowButtonStyle))
                    .into();
                
                let countdown_display = format!("{:.1}s", self.timer_countdown);
                let timer_color = if self.timer_countdown <= 3.0 && self.timer_countdown > 0.0 {
                    Color::from_rgb8(255, 100, 0)
                } else if self.timer_countdown <= 0.0 {
                    Color::from_rgb8(255, 0, 0)
                } else {
                    *BLUE_TEXT
                };


                (
                    start_btn,
                    stop_btn,
                    text("Recording... (10s max)").size(BODY_SIZE).style(iced::theme::Text::Color(*BLUE_TEXT)).into(),
                    text(countdown_display).size(HEADING_SIZE).style(iced::theme::Text::Color(timer_color)).font(MONOSPACE_FONT).into(),
                )
            },
        };

        let confirm_button_enabled = matches!(self.recording_state, RecordingState::Stopped(_));
        let confirm_button_style = if confirm_button_enabled {
            theme_fn(YellowButtonStyle)
        } else {
            theme_fn(DisabledButtonStyle)
        };
        let confirm_button = button(text("Confirm").size(BODY_SIZE))
            .on_press_maybe(if confirm_button_enabled { Some(Message::NavigateTo(AppScreen::CustomizeLeds)) } else { None })
            .padding([5,20])
            .style(confirm_button_style)
            .width(Length::Shrink);


        let bottom_buttons = row![
            back_button,
            Space::with_width(Length::Fill),
            confirm_button,
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fill);

        column![
            Space::with_height(Length::Fixed(50.0)),
            image(HEAR_ME_ICON.clone())
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0)),
            text("Hear Me")
                .size(HEADING_SIZE)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            container(
                text("Record a 10-Second Special Sound for Your Badge!")
                .size(HEADING_SIZE - 5)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(Length::Fill)
            )
            .padding([0, 50]),
            status_text_element,
            Space::new(Length::Shrink, Length::Fixed(20.0)),
            timer_text_element,
            Space::new(Length::Shrink, Length::Fixed(20.0)),
            row![
                record_button,
                Space::with_width(Length::Fixed(20.0)),
                stop_button,
            ]
            .align_items(Alignment::Center)
            .width(Length::Shrink)
            .spacing(20),
            Space::with_height(Length::Fill),
            bottom_buttons,
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn render_customize_leds_screen(&self) -> Element<Message> {
        let back_button = button(text("Back").size(BODY_SIZE))
            .on_press(Message::NavigateTo(AppScreen::RecordSound))
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


// --- Audio Event Subscription Recipe ---
struct AudioEventSubscription;

impl AudioEventSubscription {
    fn new() -> Self {
        AudioEventSubscription {}
    }
}

impl Recipe for AudioEventSubscription {
    type Output = Message;

    fn hash(&self, state: &mut iced_futures::core::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced_futures::subscription::EventStream,
    ) -> Pin<Box<dyn futures::Stream<Item = Self::Output> + Send + 'static>> {
        use tokio::sync::mpsc;

        let (cmd_tx, cmd_rx) = mpsc::channel(1);
        let (event_tx, mut event_rx) = mpsc::channel(100);

        {
            let mut sender_guard = AUDIO_CMD_SENDER_GLOBAL.lock().unwrap();
            *sender_guard = Some(cmd_tx);
        }

        let audio_buffer_task_ref = Arc::new(Mutex::new(Vec::new()));

        tokio::task::spawn_blocking(move || {
            audio_management_task_blocking(cmd_rx, event_tx, audio_buffer_task_ref);
        });

        Box::pin(async_stream::stream! {
            while let Some(event) = event_rx.recv().await {
                yield Message::AudioOutput(event);
            }
        })
    }
}
// --- Audio Recording Async Functions ---

fn get_audio_config_and_format(
    device_option: Option<cpal::Device>,
) -> Result<(StreamConfig, SampleFormat, cpal::ChannelCount), String> {
    let device = device_option.ok_or("No default input device found")?;

    let supported_configs_range = device.supported_input_configs()
        .map_err(|e| format!("Error getting supported input configs: {}", e))?;

    let (config, actual_sample_format, actual_channels) = 
        match supported_configs_range.filter(|config| {
            config.channels() == CHANNELS &&
            config.min_sample_rate().0 <= SAMPLE_RATE &&
            config.max_sample_rate().0 >= SAMPLE_RATE &&
            (config.sample_format() == SampleFormat::I16 || config.sample_format() == SampleFormat::F32)
        }).next()
    {
        Some(supported_config) => {
            let config = supported_config.with_sample_rate(cpal::SampleRate(SAMPLE_RATE));
            let actual_sample_format = config.sample_format();
            let actual_channels = config.channels();
            (config.into(), actual_sample_format, actual_channels)
        },
        None => {
            let default_config = device.default_input_config()
                .map_err(|e| format!("No ideal I16/F32 config found, failed to get default input config: {}", e))?;
            println!("Warning: No ideal I16/F32 config found. Using default config with format {:?} and sample rate {}. Data will be converted.", default_config.sample_format(), default_config.sample_rate().0);
            (default_config.config(), default_config.sample_format(), default_config.channels())
        }
    };
    Ok((config, actual_sample_format, actual_channels))
}


fn audio_management_task_blocking(
    mut cmd_receiver: tokio::sync::mpsc::Receiver<AudioCommand>,
    event_sender: tokio::sync::mpsc::Sender<AudioEvent>,
    audio_buffer: Arc<Mutex<Vec<i16>>>,
) {
    let mut active_stream: Option<Stream> = None;
    let _current_sample_format: Option<SampleFormat> = None;
    let _current_channels: Option<cpal::ChannelCount> = None;

    loop {
        let cmd = cmd_receiver.blocking_recv(); 
        
        match cmd {
            Some(AudioCommand::Start(config, sample_format, channels, _buffer_ref)) => {
                if active_stream.is_some() {
                    let _ = active_stream.take().map(|s| s.pause().ok());
                }

                let host = cpal::default_host();
                let device = match host.default_input_device() {
                    Some(d) => d,
                    None => {
                        let error_msg = "Audio task: No input device found to build stream.".to_string();
                        eprintln!("{}", error_msg);
                        let _ = event_sender.blocking_send(AudioEvent::StreamError(error_msg));
                        continue;
                    }
                };

                let error_fn = {
                    let event_sender = event_sender.clone();
                    move |err| {
                        eprintln!("Audio task: an error occurred on stream: {}", err);
                        let _ = event_sender.blocking_send(AudioEvent::StreamError(format!("{}", err)));
                    }
                };
                
                let stream_result = match sample_format {
                    SampleFormat::F32 => device.build_input_stream(
                        &config,
                        {
                            let audio_buffer_clone = Arc::clone(&audio_buffer);
                            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                                let mut buffer = audio_buffer_clone.lock().unwrap();
                                if channels == 2 {
                                    buffer.extend(data.iter().step_by(2).map(|&s| (s * i16::MAX as f32) as i16));
                                } else {
                                    buffer.extend(data.iter().map(|&s| (s * i16::MAX as f32) as i16));
                                }
                            }
                        },
                        error_fn,
                        None,
                    ),
                    SampleFormat::I16 => device.build_input_stream(
                        &config,
                        {
                            let audio_buffer_clone = Arc::clone(&audio_buffer);
                            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                                let mut buffer = audio_buffer_clone.lock().unwrap();
                                if channels == 2 {
                                    buffer.extend(data.iter().step_by(2).cloned());
                                } else {
                                    buffer.extend_from_slice(data);
                                }
                            }
                        },
                        error_fn,
                        None,
                    ),
                    SampleFormat::U16 => device.build_input_stream(
                        &config,
                        {
                            let audio_buffer_clone = Arc::clone(&audio_buffer);
                            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                                let mut buffer = audio_buffer_clone.lock().unwrap();
                                if channels == 2 {
                                    buffer.extend(data.iter().step_by(2).map(|&s| (s as i32 - i16::MAX as i32 - 1) as i16));
                                } else {
                                    buffer.extend(data.iter().map(|&s| (s as i32 - i16::MAX as i32 - 1) as i16));
                                }
                            }
                        },
                        error_fn,
                        None,
                    ),
                    _ => {
                        let error_msg = format!("Audio task: Unsupported sample format for stream building: {:?}", sample_format);
                        eprintln!("{}", error_msg);
                        let _ = event_sender.blocking_send(AudioEvent::StreamError(error_msg));
                        continue;
                    }
                };

                match stream_result {
                    Ok(stream) => {
                        if let Err(e) = stream.play() {
                            let error_msg = format!("Audio task: Failed to play stream: {}", e);
                            eprintln!("{}", error_msg);
                            let _ = event_sender.blocking_send(AudioEvent::StreamError(error_msg));
                        } else {
                            println!("Audio task: Stream started.");
                            active_stream = Some(stream);
                            let _ = event_sender.blocking_send(AudioEvent::StreamStarted);
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Audio task: Failed to build stream: {}", e);
                        eprintln!("{}", error_msg);
                        let _ = event_sender.blocking_send(AudioEvent::StreamError(error_msg));
                    }
                }
            },
            Some(AudioCommand::Stop) => {
                if let Some(stream) = active_stream.take() {
                    let _ = stream.pause().ok();
                    println!("Audio task: Stream stopped.");

                    let cloned_buffer = audio_buffer.lock().unwrap().drain(..).collect::<Vec<i16>>();
                    let temp_path = PathBuf::from("my_badge_sound.wav");
                    match save_wav_file_blocking(cloned_buffer, temp_path.clone()) {
                        Ok(_) => {
                            println!("Audio task: WAV saved successfully in blocking task.");
                            let _ = event_sender.blocking_send(AudioEvent::RecordingDataReady(temp_path));
                        },
                        Err(e) => {
                            eprintln!("Audio task: Error saving WAV: {}", e);
                            let _ = event_sender.blocking_send(AudioEvent::RecordingError(e));
                        }
                    }
                }
            },
            None => {
                println!("Audio task: Command channel closed, quitting.");
                if let Some(stream) = active_stream.take() {
                    let _ = stream.pause().ok();
                }
                return;
            },
        }
    }
}


fn save_wav_file_blocking(audio_data: Vec<i16>, path: PathBuf) -> Result<(), String> {
    let spec = WavSpec {
        channels: CHANNELS,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: BITS_PER_SAMPLE,
        sample_format: hound::SampleFormat::Int,
    };

    println!("Saving WAV file (blocking) to: {:?} with spec: {:?}", path, spec);

    let mut writer = WavWriter::create(&path, spec)
        .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

    for sample in audio_data.iter() {
        writer.write_sample(*sample)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    writer.flush().map_err(|e| format!("Failed to flush writer: {}", e))?;
    writer.finalize().map_err(|e| format!("Failed to finalize writer: {}", e))?;
    
    println!("WAV file saved successfully (blocking).");
    Ok(())
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