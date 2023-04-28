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
function set_stage(json) {
    const stage = document.getElementById("stage");
    stage.innerHTML = "";

    if (json["state"] === "PlayerJoined") {
        // Display new player name.
        const PLAYER_PREFIXES = ["pb", "pl", "pt", "pr"];

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
    } else if (Object.keys(json["state"])[0] == "MatchAborted") {
        stage.innerHTML = "<div style='color: red'>Aborted</div>";
    }
}

// Updates the info text to match the session state sent by the server.
function set_info(json) {
    const info = document.getElementById("info");
    info.innerHTML = "";

    if (json["state"] === "PlayerJoined") {
        info.innerHTML = "Waiting for other players to join";
    } else if (Object.keys(json["state"])[0] == "MatchAborted") {
        info.innerHTML = "<div style='color: red'>" + json["state"]["MatchAborted"] + "</div>";
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
        set_stage(json);
        set_info(json);

        // Pretty print hand in response JSON.
        if (json["history"] !== null && json["history"]["game_history"] !== null &&
            json["history"]["game_history"]["hand"] !== null) {
            const hand = json["history"]["game_history"]["hand"];
            for (let i = 0; i < hand.length; ++i) {
                hand[i] = pretty_card(hand[i]);
            }
        }

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
