// ignore_for_file: constant_identifier_names
import 'package:flutter_svg/flutter_svg.dart';

class WalletAssets {
  static Future<void> preloadPidSvgs() async {
    final svgs = [svg_rijks_card_holo, svg_rijks_card_bg_light, svg_rijks_card_bg_dark];
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

  // SVGS
  static const svg_rijks_card_holo = 'assets/non-free/svg/rijks_card_holo.svg';
  static const svg_rijks_card_bg_light = 'assets/non-free/svg/rijks_card_bg_light.svg';
  static const svg_rijks_card_bg_dark = 'assets/non-free/svg/rijks_card_bg_dark.svg';

  // IMAGES
  static const image_bg_diploma = 'assets/non-free/images/bg_diploma.png';
  static const image_bg_nl_driving_license = 'assets/non-free/images/bg_nl_driving_license.png';
  static const image_person_x = 'assets/non-free/images/person_x.png';
  static const image_attribute_placeholder = 'assets/non-free/images/attribute_placeholder.png';
  static const image_bg_health_insurance = 'assets/non-free/images/bg_health_insurance.png';
  static const image_intro_page_4 = 'assets/non-free/images/intro_page_4.png';
  static const image_intro_page_3 = 'assets/non-free/images/intro_page_3.png';
  static const image_intro_page_2 = 'assets/non-free/images/intro_page_2.png';
  static const image_intro_page_1 = 'assets/non-free/images/intro_page_1.png';

  // ILLUSTRATIONS
  static const illustration_sign_1 = 'assets/non-free/illustrations/sign_1.png';
  static const illustration_pin_timeout = 'assets/non-free/illustrations/pin_timeout.png';
  static const illustration_sign_2 = 'assets/non-free/illustrations/sign_2.png';
  static const illustration_general_error = 'assets/non-free/illustrations/general_error.png';
  static const illustration_no_internet_error = 'assets/non-free/illustrations/no_internet_error.png';
  static const illustration_conditions_screen = 'assets/non-free/illustrations/conditions_screen.png';
  static const illustration_placeholder_contract = 'assets/non-free/illustrations/placeholder_contract.png';
  static const illustration_forgot_pin_header = 'assets/non-free/illustrations/forgot_pin_header.png';
  static const illustration_privacy_policy_screen = 'assets/non-free/illustrations/privacy_policy_screen.png';
  static const illustration_digid_failure = 'assets/non-free/illustrations/digid_failure.png';
  static const illustration_server_error = 'assets/non-free/illustrations/server_error.png';
  static const illustration_placeholder_generic = 'assets/non-free/illustrations/placeholder_generic.png';
  static const illustration_personalize_wallet_intro = 'assets/non-free/illustrations/personalize_wallet_intro.png';

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

  // ICONS
  static const icon_first_share = 'assets/non-free/icons/first_share.png';
  static const icon_card_share = 'assets/non-free/icons/card_share.png';
}
