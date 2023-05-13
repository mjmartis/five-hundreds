// Utilities.

// Takes in a card JSON struct and returns a nicer string representation.
function pretty_card(card_json) {
    const FACES = [null, null, null, null, 4, 5, 6,
        7, 8, 9, 10, "J", "Q", "K", "A"
    ];
    const SUITS = {
        "Spades": "♠",
        "Clubs": "♣",
        "Diamonds": "◆",
        "Hearts": "♥"
    };

    if (card_json === "Joker") {
        return "★";
    }

    return FACES[card_json['SuitedCard']['face']] +
        SUITS[card_json['SuitedCard']['suit']];
}

// Takes in a bid JSON struct and returns a nicer string representation.
function pretty_bid(bid_json) {
    const SUITS = {
        "Spades": "♠",
        "Clubs": "♣",
        "Diamonds": "◆",
        "Hearts": "♥"
    };

    if (typeof(bid_json) === "string") {
        return bid_json[0];
    }

    // Now we must have a count and suit.
    const count = bid_json["Tricks"][0];
    const suit = SUITS[bid_json["Tricks"][1]["Suit"]] || "NT";

    return count + suit;
}

// Convert bid string back into the json struct expected by the server.
function ugly_bid(bid) {
    const SUITS = {
        "♠": "Spades",
        "♣": "Clubs",
        "◆": "Diamonds",
        "♥": "Hearts"
    };

    switch (bid) {
        case "P":
            return "Pass";

        case "M":
            return "Mis";

        case "O":
            return "OpenMis";

        default: {
            // Ten is the only bid that has a different number prefix size.
            const num_length = bid[1] === "0" ? 2 : 1;
            const count = parseInt(bid.slice(0, num_length));
            const pretty_suit = bid.slice(num_length);

            const suit = pretty_suit == "NT" ? "NoTrumps" : {
                "Suit": SUITS[pretty_suit]
            };

            return {
                "Tricks": [count, suit]
            };
        }
    }
}

// Helper: create a new option for a select element that contains the given
// text.
function new_select_option(contents) {
    const opt = document.createElement("option");
    opt.value = contents;
    opt.innerHTML = contents;
    return opt;
}

// Inserts our bid-picker faux element into the given div.
function insert_bid_picker(e) {
    // Dodgy: use a custom property for our bid.
    e.bid = "P";

    // First comes a drop-down selector for bid count.
    const count = document.createElement("select");
    e.appendChild(count);
    count.appendChild(new_select_option("P"));
    for (const c of [6, 7, 8, 9, 10]) {
        count.add(new_select_option(c));
    }
    count.appendChild(new_select_option("M"));
    count.appendChild(new_select_option("O"));

    // Next comes drop-down selector for suit.
    const suit = document.createElement("select");
    suit.disabled = true;
    e.appendChild(suit);
    for (const s of ["♠", "♣", "◆", "♥", "NT"]) {
        suit.add(new_select_option(s));
    }

    // Update custom property (and enable/disable suit selector) when a new
    // count option is selected.
    const d = e;
    count.onchange = function() {
        suit.disabled = isNaN(parseInt(count.value[0]));
        if (suit.disabled) {
            d.bid = count.value;
        } else {
            d.bid = count.value + suit.value;
        }
    }

    // Update custom property when a new suit is selected.
    suit.onchange = function() {
        d.bid = count.value + suit.value;
    }
}

// Updates the stage text to match the session state sent by the server.
function update_stage(json) {
    const stage = document.getElementById("stage");
    stage.innerHTML = "";

    // Some states are strings, some are objects.
    const key = Object.keys(json["state"])[0];
    const state = key == "0" ? json["state"] : key;

    switch (state) {
        case "PlayerJoined":
            stage.innerHTML = "Lobby";
            break;

        case "MatchAborted":
            // Display red aborted title.
            stage.innerHTML = "<div style='color: red'>Aborted</div>";
            break;

        case "WaitingForTheirBid":
        case "WaitingForYourBid":
            stage.innerHTML = "Bidding";
            break;
    }
}

// Updates the info text to match the session state sent by the server.
function update_info(json) {
    const info = document.getElementById("info");
    info.innerHTML = "";

    // Some states are strings, some are objects.
    const key = Object.keys(json["state"])[0];
    const state = key == "0" ? json["state"] : key;

    switch (state) {
        case "PlayerJoined":
            info.innerHTML = "Waiting for other players to join";
            break;

        case "MatchAborted":
            info.innerHTML = "<div style='color: red'>" + json["state"]["MatchAborted"] + "</div>";
            break;

        case "WaitingForYourBid":
            info.innerHTML = "Make your bid";
            break;

        case "WaitingForTheirBid":
            info.innerHTML = "Waiting for player " + (json["history"]["game_history"]["bidding_history"]["current_bidder_index"] + 1) + " to bid";
            break;
    }
}

// Updates the names around the match surface to reflect the state sent by the server.
function update_player_names(json) {
    const PLAYER_PREFIXES = ["pb", "pl", "pt", "pr"];

    // Player info is in the lobby history.
    if (json["history"] === null || json["history"]["lobby_history"] === null) {
        return;
    }

    // Display new info.
    const player_count = json["history"]["lobby_history"]["player_count"];
    const player_index = json["history"]["lobby_history"]["your_player_index"];

    for (let i = 0; i < player_count; ++i) {
        const index = (i - player_index + 4) % 4;
        const e = document.getElementById(PLAYER_PREFIXES[index] + "_name");
        e.innerHTML = "Player " + (i + 1);
        e.classList.remove("greyed");

        if (i == player_index) {
            e.style.setProperty("font-weight", "bold");
        }
    }
}

// Updates the displayed cards for all players to match those sent by the server.
function update_cards(json) {
    // Clear old info.
    document.getElementById("hand").innerHTML = "";

    // Hand info is in the game history.
    if (json["history"] === null || json["history"]["game_history"] === null ||
        json["history"]["game_history"]["hand"] === null) {
        return;
    }

    const hand = document.getElementById("hand");
    for (const details of json["history"]["game_history"]["hand"]) {
        const card = document.createElement("div");
        card.classList.add("card");
        card.innerHTML = details;

        if (details.slice(-1) === "◆" || details.slice(-1) === "♥") {
            // Red cards.
            card.style.setProperty("color", "red");
        } else if (details.slice(-1) === "★") {
            // Joker.
            card.style.setProperty("color", "blue");
        }

        hand.appendChild(card);
    }
}

function update_aux_ui(json) {
    // Clear old info.
    const bids = document.getElementById("bids");
    bids.style.setProperty("display", "none");
    for (const cell of bids.getElementsByTagName("td")) {
        cell.classList.add("greyed");
    }

    // Available bids are only shown when it's your turn to bid.
    if (json["state"] === null || json["state"]["WaitingForYourBid"] === undefined) {
        return;
    }

    // Conditionally ungrey unavailable bids.
    bids.style.setProperty("display", "block");
    const bidOptions = json["state"]["WaitingForYourBid"];
    for (const bid of bidOptions) {
        for (const cell of bids.getElementsByTagName("td")) {
            if (cell.innerHTML === bid) {
                cell.classList.remove("greyed");
            }
        }
    }
}

// Main logic.
function main() {
    renderjson.set_show_to_level("all");

    // Add collapse / uncollapse logic for the API step menu.
    const buttons = document.getElementsByClassName("collapse_button");
    for (const button of buttons) {
        button.addEventListener("click", function() {
            // Toggle our visibility.
            const content = this.nextElementSibling;
            if (content.style.display === "block") {
                content.style.display = "none";
            } else {
                content.style.display = "block";
            }

            // Hide every other button.
            for (const other of buttons) {
                if (other === this)
                    continue;

                other.nextElementSibling.style.display = "none";
            }
        });
    }

    // Insert pseudo elements for choosing bids.
    for (const e of document.getElementsByClassName("bid_picker")) {
        insert_bid_picker(e);
    }

    // Connect to the server.
    const socket = new WebSocket("ws://192.168.1.69:8080");

    socket.onmessage = (event) => {
        const json = JSON.parse(event.data);

        // Pretty print hand in response JSON. This makes API responses easier
        // for humans to parse.
        if (json["history"] !== null && json["history"]["game_history"] !== null &&
            json["history"]["game_history"]["hand"] !== null) {
            const hand = json["history"]["game_history"]["hand"];
            for (let i = 0; i < hand.length; ++i) {
                hand[i] = pretty_card(hand[i]);
            }
        }

        // Also pretty print bids in response JSON.
        if (json["state"] !== null && json["state"]["WaitingForYourBid"] !== undefined) {
            const bids = json["state"]["WaitingForYourBid"];
            for (let i = 0; i < bids.length; ++i) {
                bids[i] = pretty_bid(bids[i]);
            }
        }

        // Update client visuals.
        update_stage(json);
        update_info(json);
        update_player_names(json);
        update_cards(json);
        update_aux_ui(json);

        // Add new response to top of state log.
        document.getElementById("states").prepend(document.createElement("hr"));
        document.getElementById("states").prepend(renderjson(json));
    };

    socket.onopen = (event) => {
        // Enable step UI.
        const steps = document.getElementById("steps");
        steps.classList.remove("greyed");
    };

    // Step UI logic.

    // Send Join step.
    document.getElementById("join_button").addEventListener("click", () => {
        const payload = {
            "Join": parseInt(document.getElementById("join_team").value),
        };
        socket.send(JSON.stringify(payload));
    });

    // Send Bid step.
    document.getElementById("bid_button").addEventListener("click", () => {
        const payload = {
            "MakeBid": ugly_bid(document.getElementById("picked_bid").bid),
        };
        socket.send(JSON.stringify(payload));
    });
}

main();