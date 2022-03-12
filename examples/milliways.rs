//! A more comprehensive example of how to use `sapi-lite`.
//!
//! This example listens for TCP connections on a given address and port. Every peer that connects
//! is treated like a guest at Milliways, the fictional "Restaurant at the End of the Universe" from
//! the "Hitchhiker's Guide to the Galaxy" by Douglas Adams.
//!
//! Each guest has to identify themselves by their name, which has to be unique across all currently
//! connected peers. The guest can then order items from the restaurant's menu, or leave when they
//! are done.
//!
//! Each arrival, departure, and order are announced on the computer running the Milliways server,
//! using `sapi-lite` TTS features. The Milliways server listens to the default audio device on the
//! computer, and can recognize voice commands phrased as "serve <item> to <guest>", where <item>
//! is one of the dishes or beverages on the menu and <guest> is the name of a guest. The outcome of
//! each command is also announced using TTS.

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt::Write;
use std::sync::{Arc, Mutex, MutexGuard};

use futures::{SinkExt, StreamExt};
use sapi_lite::stt::{Grammar, Recognizer, RuleArena};
use sapi_lite::tokio::{AsyncSynthesizer, UnicastContext};
use sapi_lite::tts::SpeechBuilder;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LinesCodec};

fn main() -> Result<(), Box<dyn Error>> {
    // Bring BuilderExt into scope so enable_sapi() is available.
    use sapi_lite::tokio::BuilderExt;

    // Initialize SAPI for the main thread.
    sapi_lite::initialize()?;

    // Call enable_sapi() to ensure all tokio threads also initialize SAPI.
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .enable_sapi()
        .build()?
        .block_on(tokio_main())?;

    // Finalize SAPI for the main thread.
    sapi_lite::finalize();

    Ok(())
}

async fn tokio_main() -> Result<(), Box<dyn Error>> {
    // Prepare all we need to recognize spoken commands.
    let recognizer = Recognizer::new()?;
    let (reco_ctx, mut reco_sub) = UnicastContext::new(&recognizer, 5)?;

    // Construct the shared state. Note that we're passing the ownership of the recognition context
    // to the constructor. We only need the UnicastSubscriber to await recognized phrases, whereas
    // the shared state will need the context in order to build and load the grammar.
    let restaurant = Arc::new(Restaurant::new(reco_ctx)?);

    let listen_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6677".to_string());

    let listener = TcpListener::bind(listen_addr).await?;

    loop {
        tokio::select! {
            Ok((stream, _)) = listener.accept() => {
                // Spawn a handler for the connected guest.
                let restaurant = restaurant.clone();
                tokio::spawn(async move {
                    let _ = process_guest(restaurant, stream).await;
                });
            },
            phrase = reco_sub.recognize() => {
                // Extract the index of menu item to be served from the semantic tags in the phrase.
                let item = *phrase.semantics[0].value.as_int().unwrap() as usize;
                // Extract the name of the guest to serve from the semantic tags in the phrase.
                let guest_name = phrase.semantics[1]
                    .value
                    .as_string()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                // Serve the menu item to the guest.
                restaurant.serve_item(MENU[item], guest_name);
            },
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    Ok(())
}

/// The list of food and drink guests can order.
const MENU: [&'static str; 7] = [
    "Pan Galactic Gargle Blaster",
    "jinond-o-nicks",
    "ol' Janx Spirit",
    "Dish of the Day",
    "Vegan Rhino cutlet",
    "Algolian Zylatburger",
    "salad",
];

/// The transmitter end of the channel used to serve food and drink to a specific guest.
type GuestTx = mpsc::UnboundedSender<&'static str>;

/// The shared state of the restaurant.
struct Restaurant {
    /// The collection of guests currently dining at Milliways.
    guests: Mutex<Guests>,
    /// The context used for configuring the recognizable spoken commands.
    reco_ctx: UnicastContext,
    /// The speech synthesizer used for announcing the outcomes of spoken commands.
    synthesizer: AsyncSynthesizer,
}

/// Holds the information about the guests and the grammar that allows to serve them.
struct Guests {
    /// Maps the guest's name to their channel.
    map: HashMap<String, GuestTx>,
    /// Holds the recognition grammar, or `None` if there are no guests.
    grammar: Option<Grammar>,
}

impl Restaurant {
    /// Create a new instance.
    fn new(reco_ctx: UnicastContext) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            guests: Mutex::new(Guests {
                map: HashMap::new(),
                grammar: None,
            }),
            reco_ctx,
            synthesizer: AsyncSynthesizer::new()?,
        })
    }

    /// Add a guest if their name is unique, otherwise give their channel back to the caller.
    fn add_guest(&self, name: &str, guest: GuestTx) -> Result<(), GuestTx> {
        let mut guests = self.guests.lock().unwrap();

        if guests.map.contains_key(name) {
            return Err(guest);
        }

        guests.map.insert(name.to_string(), guest);
        self.update_grammar(guests);

        Ok(())
    }

    /// Remove the guest who departed.
    fn remove_guest(&self, name: &str) {
        let mut guests = self.guests.lock().unwrap();
        guests.map.remove(name);
        self.update_grammar(guests);
    }

    /// Find the guest with the given name and, if present, send the item over their channel.
    fn serve_item(&self, item: &'static str, name: String) {
        let mut speech = SpeechBuilder::new();
        {
            let guests = self.guests.lock().unwrap();
            match guests.map.get(&name) {
                Some(guest) => {
                    // Send the item to be served to the guest.
                    let _ = guest.send(item);

                    write!(speech, "{} has been served to {}.", &item, name).unwrap();
                }
                None => {
                    write!(speech, "There is no one called {} here.", name).unwrap();
                }
            }
        }

        // Announce the outcome.
        self.synthesizer.speak_and_forget(speech).unwrap();
    }

    /// Update the recognition grammar to reflect the list of guests.
    fn update_grammar(&self, mut guests: MutexGuard<Guests>) {
        // If we had an active grammar, deactivate it.
        if let Some(old_grammar) = guests.grammar.take() {
            old_grammar.set_enabled(false).unwrap();
        }

        // If we don't have any guests right now, we don't need a grammar
        if guests.map.is_empty() {
            return;
        }

        // Since the grammar is a graph of rules referring to each other, we need to allocate the
        // rules somewhere. We can use a `RuleArena` for that.
        let arena = RuleArena::new();

        let mut name_choices = Vec::new();
        for (name, _) in guests.map.iter() {
            name_choices.push(arena.semantic(name.as_str(), arena.text(name)));
        }

        let mut item_choices = Vec::new();
        for item in 0..MENU.len() {
            item_choices.push(arena.semantic(item as i32, arena.text(MENU[item])));
        }

        // Add the top level rule and build the grammar.
        let grammar = self
            .reco_ctx
            .grammar_builder()
            .add_rule(arena.sequence(vec![
                arena.text("serve"),
                arena.choice(item_choices),
                arena.text("to"),
                arena.choice(name_choices),
            ]))
            .build()
            .unwrap();

        // Activate the grammar, so its phrases can be recognized.
        grammar.set_enabled(true).unwrap();

        // Store the grammar.
        guests.grammar = Some(grammar);
    }
}

/// Communicate with the guest.
async fn process_guest<'a>(
    restaurant: Arc<Restaurant>,
    stream: TcpStream,
) -> Result<(), Box<dyn Error>> {
    // Create a speech synthesizer to announce guest's arrival, orders, and departure.
    let synthesizer = AsyncSynthesizer::new().unwrap();

    // Create a channel to receive the food and drink.
    let (mut tx, mut rx) = mpsc::unbounded_channel();

    let mut lines = Framed::new(stream, LinesCodec::new());

    // Greet the guest.
    lines
        .send(
            "Welcome to Welcome to Milliways, the Restaurant at the End of the Universe!\r\n\
            May I have your name, please?",
        )
        .await?;

    // Each guest must have a unique name to avoid ambiguities in the speech recognition grammar.
    let name = loop {
        // Get the name.
        let name = match lines.next().await {
            Some(Ok(line)) => line.trim().to_string(),
            _ => {
                return Ok(());
            }
        };

        // If the name is not unique, ask again.
        if let Err(err_tx) = restaurant.add_guest(&name, tx) {
            lines
                .send(
                    "Hmm, it would seem you are already here tonight.\r\n\
                    Meeting yourself is impossible at Milliways.\r\n\
                    Most likely you are misremembering who you are.\r\n\
                    It is not unusual for our customers to be disoriented by the time journey.\r\n\
                    Take your time.\r\n\
                    When you are feeling up to it, please let me know your name.",
                )
                .await?;
            tx = err_tx;
        } else {
            break name;
        }
    };

    // Announce the guest's arrival.
    synthesizer
        .speak(format!("{} has arrived", &name))
        .await
        .unwrap();

    // Show the menu and instructions.
    lines
        .send("Excellent! Your reservation seems to be in order.")
        .await?;
    send_menu(&mut lines).await?;
    lines
        .send("To go back to your own timeline, just type in 'leave'.")
        .await?;

    loop {
        tokio::select! {
            line = lines.next() => match line {
                Some(Ok(command)) => {
                    if command == "leave" {
                        break;
                    } else if command == "menu" {
                        send_menu(&mut lines).await?;
                    } else if let Some(arg) = command.strip_prefix("order ") {
                        // Is the guest's order on the menu?
                        if let Some(item) = find_item_in_menu(arg) {
                            // Announce the guest's order.
                            synthesizer.speak(format!("{} ordered {}", name, item)).await?;
                        }
                    }
                },
                _ => break
            },
            event = rx.recv() => match event {
                // We've received something to serve to the guest.
                Some(item) => {
                    lines.send(format!("Here's your {}. Share and enjoy!", item)).await?;
                },
                _ => {
                    return Ok(());
                }
            }
        }
    }

    // Remove the departed guest and update the grammar.
    restaurant.remove_guest(&name);

    // Announce the guest's departure.
    synthesizer
        .speak(format!("{} has left", &name))
        .await
        .unwrap();

    Ok(())
}

/// Send the menu to the guest and explain how to order something from it.
async fn send_menu(lines: &mut Framed<TcpStream, LinesCodec>) -> Result<(), Box<dyn Error>> {
    lines.feed("Our menu for today is:").await?;
    for item in MENU.iter() {
        lines.feed(item).await?;
    }
    lines
        .send(
            "To order anything from the menu, type in 'order <item>'.\r\n\
            For example: order Pan Galactic Gargle Blaster\r\n\
            To see the menu again, type in 'menu'.",
        )
        .await?;
    Ok(())
}

/// Check if the requested item is on the menu. If so, return the canonical name for it.
fn find_item_in_menu(text: &str) -> Option<&'static str> {
    for item in MENU.iter() {
        if text.eq_ignore_ascii_case(item) {
            return Some(item);
        }
    }
    None
}
