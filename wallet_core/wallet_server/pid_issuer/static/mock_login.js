// On the first card submission, disable every card so a slow POST doesn't invite repeat clicks, and
// reveal the loading overlay. The submitting form still navigates as usual.
(function () {
    const cards = document.querySelectorAll(".card");
    const overlay = document.getElementById("overlay");
    cards.forEach(function (form) {
        form.addEventListener("submit", function () {
            cards.forEach(function (other) {
                other.querySelector("button").disabled = true;
            });
            if (overlay) {
                overlay.hidden = false;
            }
        });
    });
})();
