import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../../data/store/active_locale_provider.dart';
import '../../../../util/extension/build_context_extension.dart';

class TitleText extends StatelessWidget {
  final String data;
  final TextStyle? style;
  final TextAlign? textAlign;

  const TitleText(this.data, {this.style, this.textAlign, super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      header: true,
      container: true /* make sure it's always an individual semantics node */,
      child: SizedBox(
        width: double.infinity,
        child: Text.rich(
          TextSpan(text: data, locale: context.read<ActiveLocaleProvider>().activeLocale),
          style: style ?? context.textTheme.displayMedium,
          textAlign: textAlign ?? TextAlign.start,
        ),
      ),
    );
  }
}
