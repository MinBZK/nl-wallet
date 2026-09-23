// On the first card submission, disable every card so a slow POST doesn't invite repeat clicks, and
// reveal the loading overlay. The submitting form still navigates as usual.
(function () {
    const forms = document.querySelectorAll(".grid form");
    const overlay = document.getElementById("overlay");
    forms.forEach(function (form) {
        form.addEventListener("submit", function () {
            forms.forEach(function (other) {
                other.querySelector("button").disabled = true;
            });
            if (overlay) {
                overlay.hidden = false;
            }
        });
    });
})();
