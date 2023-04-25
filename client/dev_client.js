var buttons = document.getElementsByClassName("collapse_button");
console.log(buttons);

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
