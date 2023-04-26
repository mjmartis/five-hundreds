// Utilities.

// Takes in a card JSON struct and returns a nicer string representation.
function pretty_card(card_json) {
    const FACES = [null, null, null, null, " 4", " 5", " 6",
        " 7", " 8", " 9", "10", " J", " Q", " K", " A"
    ];
    const SUITS = {
        "Spades": "♠",
        "Clubs": "♣",
        "Diamonds": "◆",
        "Hearts": "♥"
    };

    if (card_json === "Joker") {
        return "★★";
    }

    return FACES[card_json['SuitedCard']['face']] +
        SUITS[card_json['SuitedCard']['suit']];
}

// Main logic.

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

    switch (Object.keys(json["state"])[0]) {
        case "MatchAborted":
            document.getElementById("stage").innerHTML = "<div style='color: red'>Aborted</div>";
            document.getElementById("info").innerHTML = "<div style='color: red'>" + json["state"]["MatchAborted"] + "</div>";
            break;
    }

    // Pretty print hand in response JSON.
    if (json["history"] !== null && json["history"]["game_history"] !== null &&
        json["history"]["game_history"]["hand"] !== null) {
        console.log(json["history"]["game_history"]["hand"]);
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
