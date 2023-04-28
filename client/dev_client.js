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

    // Clear old info.
    for (const pref of PLAYER_PREFIXES) {
        document.getElementById(pref + "_name").innerHTML = "";
    }

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
        e.style.setProperty("color", "black");

        if (i == player_index) {
            e.style.setProperty("font-weight", "bold");
        }
    }
}

// Updates the displayed cards for all players to match those sent by the server.
function update_cards(json) {
    const PLAYER_PREFIXES = ["pb", "pl", "pt", "pr"];

    // Clear old info.
    for (const pref of PLAYER_PREFIXES) {
        document.getElementById(pref + "_hand").innerHTML = "";
    }

    // Hand info is in the game history.
    if (json["history"] === null || json["history"]["game_history"] === null ||
        json["history"]["game_history"]["hand"] === null) {
        return;
    }

    // First, show the player's hand.
    const pb_hand = document.getElementById("pb_hand");
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

        pb_hand.appendChild(card);
    }

    // Next, show the backs of everyone else's cards.
    // TODO: account for number of cards played.
    // TODO: fix.
    for (const pref of PLAYER_PREFIXES) {
        if (pref === "pb") {
            continue;
        }

        const vert = pref == "pl" || pref == "pr";
        const hand = document.getElementById(pref + "_hand");
        for (let i = 0; i < 10; ++i) {
            const card = document.createElement("div");
            card.innerHTML = "&nbsp;&nbsp;&nbsp;";
            card.classList.add(vert ? "v_card" : "card");
            hand.appendChild(card);
            hand.appendChild(document.createElement("br"));
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

        // Update client visuals.
        update_stage(json);
        update_info(json);
        update_player_names(json);
        update_cards(json);

        // Add new response to top of state log.
        document.getElementById("states").prepend(document.createElement("hr"));
        document.getElementById("states").prepend(renderjson(json));
    };

    socket.onopen = (event) => {
        // Enable step UI.
        const steps = document.getElementById("steps");
        steps.style.setProperty("pointer-events", "auto");
        steps.style.setProperty("opacity", 1.0);
    };

    // Step UI logic.

    // Send Join step.
    document.getElementById("join_button").addEventListener("click", () => {
        const payload = {
            "Join": parseInt(document.getElementById("join_team").value),
        };
        socket.send(JSON.stringify(payload));
    });

}

main();
