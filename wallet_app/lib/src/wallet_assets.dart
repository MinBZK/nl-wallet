// ignore_for_file: constant_identifier_names
import 'package:flutter_svg/flutter_svg.dart';

class WalletAssets {
  WalletAssets._();

  static Future<void> preloadPidSvgs() async {
    final svgs = [
      svg_rijks_card_holo,
      svg_rijks_card_bg_light,
      svg_rijks_card_bg_dark,
      svg_qr_button,
      svg_qr_button_focused,
      svg_qr_button_focused_dark,
    ];
    final loaders = svgs.map(SvgAssetLoader.new);
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
  static const svg_qr_button_focused = 'assets/non-free/svg/qr_button_focused.svg';
  static const svg_qr_button_focused_dark = 'assets/non-free/svg/qr_button_focused_dark.svg';
  static const svg_app_store = 'assets/non-free/svg/app_store_logo.svg';
  static const svg_play_store = 'assets/non-free/svg/play_store_logo.svg';

  static const svg_blocked_final = 'assets/non-free/svg/NL_WALLET_blocked_final.svg';
  static const svg_blocked_temporary = 'assets/non-free/svg/NL_WALLET_blocked_temporary.svg';
  static const svg_digid = 'assets/non-free/svg/NL_WALLET_DigiD.svg';
  static const svg_error_config_update = 'assets/non-free/svg/NL_WALLET_error_config_update.svg';
  static const svg_error_general = 'assets/non-free/svg/NL_WALLET_error_general.svg';
  static const svg_error_no_internet = 'assets/non-free/svg/NL_WALLET_error_no_internet.svg';
  static const svg_error_rooted = 'assets/non-free/svg/NL_WALLET_error_rooted.svg';
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
  static const svg_terms = 'assets/non-free/svg/NL_WALLET_terms.svg'; // Not used at the moment due to PVW-4092
  static const svg_placeholder = 'assets/non-free/svg/NL_WALLET_placeholder.svg';
  static const svg_biometrics_finger = 'assets/non-free/svg/NL_WALLET_biometrics_finger.svg';
  static const svg_biometrics_face = 'assets/non-free/svg/NL_WALLET_biometrics_face.svg';
  static const svg_update_app = 'assets/non-free/svg/NL_WALLET_update_android.svg';
  static const svg_update_app_ios = 'assets/non-free/svg/NL_WALLET_update_ios.svg';
  static const svg_tour_icon = 'assets/non-free/svg/NL_WALLET_tour_icon.svg';
  static const svg_phone = 'assets/non-free/svg/NL_WALLET_phone.svg';
  static const svg_no_cards = 'assets/non-free/svg/NL_WALLET_no_card.svg';

  static const svg_icon_face_id = 'assets/non-free/svg/icon_face_id.svg';

  // IMAGES
  static const image_tour_video_thumb_1_nl = 'assets/non-free/images/tour_video_thumb_1_nl.png';
  static const image_tour_video_thumb_1_en = 'assets/non-free/images/tour_video_thumb_1_en.png';
  static const image_tour_video_thumb_2_nl = 'assets/non-free/images/tour_video_thumb_2_nl.png';
  static const image_tour_video_thumb_2_en = 'assets/non-free/images/tour_video_thumb_2_en.png';
  static const image_tour_video_thumb_3_nl = 'assets/non-free/images/tour_video_thumb_3_nl.png';
  static const image_tour_video_thumb_3_en = 'assets/non-free/images/tour_video_thumb_3_en.png';
  static const image_tour_video_thumb_4_nl = 'assets/non-free/images/tour_video_thumb_4_nl.png';
  static const image_tour_video_thumb_4_en = 'assets/non-free/images/tour_video_thumb_4_en.png';
  static const image_tour_video_thumb_5_nl = 'assets/non-free/images/tour_video_thumb_5_nl.png';
  static const image_tour_video_thumb_5_en = 'assets/non-free/images/tour_video_thumb_5_en.png';
  static const image_tour_video_thumb_6_nl = 'assets/non-free/images/tour_video_thumb_6_nl.png';
  static const image_tour_video_thumb_6_en = 'assets/non-free/images/tour_video_thumb_6_en.png';
  static const image_tour_video_thumb_7_nl = 'assets/non-free/images/tour_video_thumb_7_nl.png';
  static const image_tour_video_thumb_7_en = 'assets/non-free/images/tour_video_thumb_7_en.png';
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
