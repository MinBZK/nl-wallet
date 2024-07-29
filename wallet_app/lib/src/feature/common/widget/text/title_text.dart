import 'package:flutter/material.dart';

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
        child: Text(
          data,
          style: style ?? context.textTheme.displayMedium,
          textAlign: textAlign,
        ),
      ),
    );
  }
}
