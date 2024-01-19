import 'package:fimber/fimber.dart';
import 'package:url_launcher/url_launcher.dart';

Future<bool> launchUrlStringCatching(String url, {LaunchMode mode = LaunchMode.platformDefault}) async {
  try {
    return launchUrl(Uri.parse(url), mode: mode);
  } catch (ex) {
    Fimber.e('Failed to launch url: $url', ex: ex);
    return false;
  }
}

Future<bool> launchUriCatching(Uri uri, {LaunchMode mode = LaunchMode.platformDefault}) async {
  try {
    return launchUrl(uri, mode: mode);
  } catch (ex) {
    Fimber.e('Failed to launch url: $uri', ex: ex);
    return false;
  }
}
