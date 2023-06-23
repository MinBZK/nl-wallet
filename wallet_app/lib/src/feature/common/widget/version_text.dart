import 'package:flutter/material.dart';
import 'package:package_info_plus/package_info_plus.dart';

import '../../../util/extension/build_context_extension.dart';

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
            context.l10n.generalVersionText(data.version, data.buildNumber),
            style: context.textTheme.bodyMedium,
          );
        } else {
          return const SizedBox.shrink();
        }
      },
    );
  }
}
