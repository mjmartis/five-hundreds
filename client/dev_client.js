const buttons = document.getElementsByClassName("collapse_button");

// Add collapse / uncollapse logic for the API step menu.
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
	// Add new response to top of state log.
	document.getElementById("states").appendChild(renderjson(JSON.parse(event.data)));
};

socket.onopen = (event) => {
	// Enable step UI.
	const steps = document.getElementById("steps");
	steps.style.setProperty("pointer-events", "auto");
	steps.style.setProperty("opacity", 1.0);

	socket.send('{"Join": 0}');
};

