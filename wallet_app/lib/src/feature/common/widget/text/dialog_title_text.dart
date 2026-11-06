import 'package:flutter/material.dart';

import 'title_text.dart';

/// [TitleText] for an [AlertDialog.title].
class DialogTitleText extends StatelessWidget {
  final String data;

  const DialogTitleText(this.data, {super.key});

  @override
  Widget build(BuildContext context) {
    // TitleText defaults to the localized textTheme style, whose even leading would shift the
    // title compared to the dialog's own titleTextStyle.
    return TitleText(data, style: DialogTheme.of(context).titleTextStyle);
  }
}
