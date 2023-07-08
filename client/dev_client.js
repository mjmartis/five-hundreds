// The JS logic for the hacky development client. Made using most-expedient
// practices, so please don't judge the quality of the code.

// Utilities.

// Accepts a JSON dict and a list of nested fields to traverse. Returns the
// final nested field if it exists, or null otherwise.
function inner_field(json, fields) {
    let cur_obj = json;
    for (const i of fields) {
        if (cur_obj[i]) {
            cur_obj = cur_obj[i];
        } else {
            return null;
        }
    }

    return cur_obj;
}

// Takes in a card JSON struct and returns a nicer string representation.
function prettyCard(cardJson) {
    const FACES = [null, null, null, null, 4, 5, 6,
        7, 8, 9, 10, "J", "Q", "K", "A"
    ];
    const SUITS = {
        "Spades": "♠",
        "Clubs": "♣",
        "Diamonds": "◆",
        "Hearts": "♥"
    };

    if (cardJson === "Joker") {
        return "★";
    }

    return FACES[cardJson['SuitedCard']['face']] +
        SUITS[cardJson['SuitedCard']['suit']];
}

// Takes in a bid JSON struct and returns a nicer string representation.
function prettyBid(bidJson) {
    const SUITS = {
        "Spades": "♠",
        "Clubs": "♣",
        "Diamonds": "◆",
        "Hearts": "♥"
    };

    if (typeof(bidJson) === "string") {
        return bidJson[0];
    }

    // Now we must have a count and suit.
    const count = bidJson["Tricks"][0];
    const suit = SUITS[bidJson["Tricks"][1]["Suit"]] || "NT";

    return count + suit;
}

// Convert bid string back into the json struct expected by the server.
function uglyBid(bid) {
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
            const numLength = bid[1] === "0" ? 2 : 1;
            const count = parseInt(bid.slice(0, numLength));
            const prettySuit = bid.slice(numLength);

            const suit = prettySuit == "NT" ? "NoTrumps" : {
                "Suit": SUITS[prettySuit]
            };

            return {
                "Tricks": [count, suit]
            };
        }
    }
}

// Helper: create a new option for a select element that contains the given
// text.
function newSelectOption(contents) {
    const opt = document.createElement("option");
    opt.value = contents;
    opt.innerHTML = contents;
    return opt;
}

// Inserts our bid-picker faux element into the given div.
function insertBidPicker(e) {
    // Dodgy: use a custom property for our bid.
    e.bid = "P";

    // First comes a drop-down selector for bid count.
    const count = document.createElement("select");
    e.appendChild(count);
    count.appendChild(newSelectOption("P"));
    for (const c of [6, 7, 8, 9, 10]) {
        count.add(newSelectOption(c));
    }
    count.appendChild(newSelectOption("M"));
    count.appendChild(newSelectOption("O"));

    // Next comes drop-down selector for suit.
    const suit = document.createElement("select");
    suit.disabled = true;
    e.appendChild(suit);
    for (const s of ["♠", "♣", "◆", "♥", "NT"]) {
        suit.add(newSelectOption(s));
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
function updateStage(json) {
    const stage = document.getElementById("stage");
    stage.innerHTML = "";

    switch (json["state"]) {
        case "PlayerJoined":
            stage.innerHTML = "Lobby";
            break;

        case "WaitingForTheirBid":
        case "WaitingForYourBid":
            stage.innerHTML = "Bidding";
            break;

        case "WaitingForTheirKitty":
        case "WaitingForYourKitty":
            stage.innerHTML = "Waiting for kitty";
            break;

        case "WaitingForTheirPlay":
        case "WaitingForYourPlay":
            // TODO: update this to use info in history struct.
            stage.innerHTML = "Playing";
            break;

        case "Error":
            // Display in red.
            stage.innerHTML = "<div style='color: red'>Error</div>";
            break;

        case "Excluded":
            // Display in red.
            stage.innerHTML = "<div style='color: red'>Excluded</div>";
            break;

        case "MatchAborted":
            // Display in red.
            stage.innerHTML = "<div style='color: red'>Aborted</div>";
            break;
    }
}

// Updates the info text to match the session state sent by the server.
function updateInfo(json) {
    const info = document.getElementById("info");
    info.innerHTML = "";

    // If there's any error, display it in red.
    const err_text = inner_field(json, ["history", "error"]);
    if (err_text) {
        info.innerHTML = "<div style='color: red'>" + err_text + "</div>";
        return;
    }

    switch (json["state"]) {
        case "PlayerJoined":
            info.innerHTML = "Waiting for other players to join";
            break;

        case "WaitingForYourBid":
            info.innerHTML = "Make your bid";
            break;

        case "WaitingForTheirBid":
            info.innerHTML = "Waiting for player " + (json["history"]["game_history"]["bidding_history"]["current_bidder_index"] + 1) + " to bid";
            break;

        case "WaitingForYourKitty":
            info.innerHTML = "Use the kitty";
            break;

        case "WaitingForTheirKitty":
            info.innerHTML = "Waiting for player " + (json["history"]["game_history"]["winning_bid_history"]["winning_bidder_index"] + 1) + " to use the kitty";
            break;

        case "WaitingForTheirPlay":
            // TODO: update to use info in history struct.
            info.innerHTML = "Waiting for player to play";
            break;

        case "Excluded":
            info.innerHTML = "<div style='color: red'>" + json["history"]["excluded_reason"] + "</div>";
            break;

        case "MatchAborted":
            info.innerHTML = "<div style='color: red'>" + json["history"]["match_history"]["match_aborted_reason"] + "</div>";
            break;
    }
}

// Updates the names around the match surface to reflect the state sent by the server.
function updatePlayerNames(json) {
    const PLAYER_PREFIXES = ["pb", "pl", "pt", "pr"];

    // Player info is in the lobby history.
    const lobbyHistory = inner_field(json, ["history", "lobby_history"]);
    if (!lobbyHistory) {
        return;
    }

    // Display new info.
    const playerCount = lobbyHistory["player_count"];
    const playerIndex = lobbyHistory["your_player_index"];

    for (let i = 0; i < playerCount; ++i) {
        const index = (i - playerIndex + 4) % 4;
        const e = document.getElementById(PLAYER_PREFIXES[index] + "_name");
        e.innerHTML = "Player " + (i + 1);
        e.classList.remove("greyed");

        if (i == playerIndex) {
            e.style.setProperty("font-weight", "bold");
        }
    }
}

// Updates the displayed cards for all players to match those sent by the server.
function updateCards(json) {
    // Clear old info.
    document.getElementById("hand").innerHTML = "";

    // Hand info is in the game history.
    const hand_info = inner_field(json, ["history", "game_history", "hand"]);
    if (!hand_info) {
        return;
    }

    // Append the kitty if it is available.
    const kitty = inner_field(json, ["history", "game_history", "winning_bid_history", "kitty"]);
    if (kitty) {
        hand_info.push(...kitty);
    }

    const hand = document.getElementById("hand");
    for (const details of hand_info) {
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

function updateAuxUi(json) {
    // Clear old info.
    const bids = document.getElementById("bids");
    bids.style.setProperty("display", "none");
    for (const cell of bids.getElementsByTagName("td")) {
        cell.classList.add("greyed");
    }

    // Available bids are only shown when it's your turn to bid.
    const bidOptions = inner_field(json, ["history", "game_history", "bidding_history", "bid_options"]);
    if (!bidOptions) {
        return;
    }

    // Conditionally ungrey unavailable bids.
    bids.style.setProperty("display", "block");
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
        insertBidPicker(e);
    }

    // Connect to the server.
    const socket = new WebSocket("ws://192.168.1.69:8080");

    socket.onmessage = (event) => {
        const json = JSON.parse(event.data);

        // Pretty print hand in response JSON. This makes API responses easier
        // for humans to parse.
        const hand = inner_field(json, ["history", "game_history", "hand"]);
        if (hand) {
            for (let i = 0; i < hand.length; ++i) {
                hand[i] = prettyCard(hand[i]);
            }
        }

        // Pretty print kitty.
        const kitty = inner_field(json, ["history", "game_history", "winning_bid_history", "kitty"]);
        if (kitty) {
            for (let i = 0; i < kitty.length; ++i) {
                kitty[i] = prettyCard(kitty[i]);
            }
        }

        // Also pretty print bids in response JSON.
        const bids = inner_field(json, ["history", "game_history", "bidding_history", "bid_options"]);
        if (bids) {
            for (let i = 0; i < bids.length; ++i) {
                bids[i] = prettyBid(bids[i]);
            }
        }

        // Update client visuals.
        updateStage(json);
        updateInfo(json);
        updatePlayerNames(json);
        updateCards(json);
        updateAuxUi(json);

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
            "MakeBid": uglyBid(document.getElementById("picked_bid").bid),
        };
        socket.send(JSON.stringify(payload));
    });
}

main();
