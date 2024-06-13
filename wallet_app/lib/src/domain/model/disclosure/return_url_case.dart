/// Helper class so that we can switch on all the possible combinations of the isLoginFlow | hasReturnUrl
enum ReturnUrlCase {
  returnUrl,
  noReturnUrl,
  loginReturnUrl,
  loginNoReturnUrl;

  static ReturnUrlCase resolve({required bool isLoginFlow, required bool hasReturnUrl}) {
    if (hasReturnUrl) {
      // Return url
      return isLoginFlow ? ReturnUrlCase.loginReturnUrl : ReturnUrlCase.returnUrl;
    } else {
      // No return url
      return isLoginFlow ? ReturnUrlCase.loginNoReturnUrl : ReturnUrlCase.noReturnUrl;
    }
  }
}
