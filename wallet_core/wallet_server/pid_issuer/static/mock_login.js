// On the first card submission, disable every card so a slow POST doesn't invite repeat clicks, and
// reveal the loading overlay. The submitting form still navigates as usual. This covers both the
// preselectable cards and the custom-BSN card; the latter only fires `submit` once its input passes
// the browser's constraint validation, so an empty/invalid BSN won't trip the overlay.
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
