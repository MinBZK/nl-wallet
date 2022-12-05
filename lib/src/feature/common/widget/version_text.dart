import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:package_info_plus/package_info_plus.dart';

class VersionText extends StatelessWidget {
  const VersionText({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<PackageInfo>(
      future: PackageInfo.fromPlatform(),
      builder: (context, snapshot) {
        if (snapshot.hasData) {
          final data = snapshot.data!;
          return Text(
            AppLocalizations.of(context).generalVersionText(data.version, data.buildNumber),
            style: Theme.of(context).textTheme.bodyText2,
          );
        } else {
          return const SizedBox.shrink();
        }
      },
    );
  }
}
