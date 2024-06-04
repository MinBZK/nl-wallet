// ignore_for_file: constant_identifier_names
import 'package:flutter_svg/flutter_svg.dart';

class WalletAssets {
  static Future<void> preloadPidSvgs() async {
    final svgs = [svg_rijks_card_holo, svg_rijks_card_bg_light, svg_rijks_card_bg_dark, svg_qr_button];
    final loaders = svgs.map((svg) => SvgAssetLoader(svg));
    await Future.wait(
      loaders.map(
        (loader) => svg.cache.putIfAbsent(
          loader.cacheKey(null),
          () => loader.loadBytes(null),
        ),
      ),
    );
  }

  // LOTTIE
  static const lottie_intro_1 = 'assets/non-free/lottie/1_WELKOM.json';
  static const lottie_intro_2 = 'assets/non-free/lottie/2_VEILIG_INLOGGEN.json';
  static const lottie_intro_3 = 'assets/non-free/lottie/3_EENVOUDIG_DELEN.json';

  // SVGS
  static const svg_rijks_card_holo = 'assets/non-free/svg/rijks_card_holo.svg';
  static const svg_rijks_card_bg_light = 'assets/non-free/svg/rijks_card_bg_light.svg';
  static const svg_rijks_card_bg_dark = 'assets/non-free/svg/rijks_card_bg_dark.svg';
  static const svg_qr_button = 'assets/non-free/svg/qr_button.svg';

  static const svg_blocked_final = 'assets/non-free/svg/NL_WALLET_blocked_final.svg';
  static const svg_blocked_temporary = 'assets/non-free/svg/NL_WALLET_blocked_temporary.svg';
  static const svg_digid = 'assets/non-free/svg/NL_WALLET_DigiD.svg';
  static const svg_error_config_update = 'assets/non-free/svg/NL_WALLET_error_config_update.svg';
  static const svg_error_general = 'assets/non-free/svg/NL_WALLET_error_general.svg';
  static const svg_error_no_internet = 'assets/non-free/svg/NL_WALLET_error_no_internet.svg';
  static const svg_error_server_outage = 'assets/non-free/svg/NL_WALLET_error_server_outage.svg';
  static const svg_error_server_overload = 'assets/non-free/svg/NL_WALLET_error_server_overload.svg';
  static const svg_error_session_expired = 'assets/non-free/svg/NL_WALLET_error_session_expired.svg';
  static const svg_pin_forgot = 'assets/non-free/svg/NL_WALLET_PIN_forgot.svg';
  static const svg_pin_set = 'assets/non-free/svg/NL_WALLET_PIN_set.svg';
  static const svg_privacy = 'assets/non-free/svg/NL_WALLET_privacy.svg';
  static const svg_sharing_failed = 'assets/non-free/svg/NL_WALLET_sharing_failed.svg';
  static const svg_sharing_success = 'assets/non-free/svg/NL_WALLET_sharing_success.svg';
  static const svg_signed = 'assets/non-free/svg/NL_WALLET_signed.svg';
  static const svg_stopped = 'assets/non-free/svg/NL_WALLET_stopped.svg';
  static const svg_terms = 'assets/non-free/svg/NL_WALLET_terms.svg';
  static const svg_placeholder = 'assets/non-free/svg/NL_WALLET_placeholder.svg';

  // IMAGES
  static const image_bg_diploma = 'assets/non-free/images/bg_diploma.png';
  static const image_bg_nl_driving_license = 'assets/non-free/images/bg_nl_driving_license.png';
  static const image_bg_health_insurance = 'assets/non-free/images/bg_health_insurance.png';

  // ILLUSTRATIONS
  static const illustration_sign_1 = 'assets/non-free/illustrations/sign_1.png';
  static const illustration_sign_2 = 'assets/non-free/illustrations/sign_2.png';
  static const illustration_placeholder_contract = 'assets/non-free/illustrations/placeholder_contract.png';
  static const illustration_digid_failure = 'assets/non-free/illustrations/digid_failure.png';
  static const illustration_placeholder_generic = 'assets/non-free/illustrations/placeholder_generic.png';

  // LOGOS
  static const logo_sign_provider = 'assets/non-free/logos/sign_provider.png';
  static const logo_wallet = 'assets/non-free/logos/wallet.png';
  static const logo_card_rijksoverheid = 'assets/non-free/logos/card_rijksoverheid.png';
  static const logo_ecommerce = 'assets/non-free/logos/ecommerce.png';
  static const logo_car_rental = 'assets/non-free/logos/car_rental.png';
  static const logo_rijksoverheid = 'assets/non-free/logos/rijksoverheid.png';
  static const logo_housing_corp = 'assets/non-free/logos/housing_corp.png';
  static const logo_monkeybike = 'assets/non-free/logos/monkeybike.png';
  static const logo_den_haag = 'assets/non-free/logos/den_haag.png';
  static const logo_nl_driving_license = 'assets/non-free/logos/nl_driving_license.png';
  static const logo_nl_health_insurance = 'assets/non-free/logos/nl_health_insurance.png';
  static const logo_digid = 'assets/non-free/logos/digid.png';
  static const logo_digid_large = 'assets/non-free/logos/digid_large.png';
  static const logo_bank = 'assets/non-free/logos/bank.png';
  static const logo_first_aid = 'assets/non-free/logos/first_aid.png';
  static const logo_rijksoverheid_label = 'assets/non-free/logos/rijksoverheid_label.png';
  static const logo_bar = 'assets/non-free/logos/bar.png';
  static const logo_zorgverzekeraar_z = 'assets/non-free/logos/zorgverzekeraar_z.png';
  static const logo_delft = 'assets/non-free/logos/delft.png';
  static const logo_rdw = 'assets/non-free/logos/rdw.png';
  static const logo_rp_placeholder = 'assets/non-free/logos/rp_placeholder.png';

  // ICONS
  static const icon_card_share = 'assets/non-free/icons/card_share.png';
}
