import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../util/mapper/card/attribute/card_attribute_label_mapper.dart';
import '../util/mapper/card/attribute/card_attribute_mapper.dart';
import '../util/mapper/card/attribute/card_attribute_value_mapper.dart';
import '../util/mapper/card/card_mapper.dart';
import '../util/mapper/card/card_subtitle_mapper.dart';
import '../util/mapper/pid/core_pid_attribute_mapper.dart';
import '../util/mapper/pid/mock_pid_attribute_mapper.dart';
import '../util/mapper/pid/pid_attribute_mapper.dart';
import '../wallet_core/error/core_error_mapper.dart';

class WalletMapperProvider extends StatelessWidget {
  final Widget child;
  final bool provideMocks;

  const WalletMapperProvider({required this.child, this.provideMocks = false, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<CoreErrorMapper>(
          create: (context) => CoreErrorMapper(),
        ),
        RepositoryProvider<PidAttributeMapper>(
          create: (context) => (provideMocks ? MockPidAttributeMapper() : CorePidAttributeMapper()),
        ),
        RepositoryProvider<CardAttributeLabelMapper>(
          create: (context) => CardAttributeLabelMapper(),
        ),
        RepositoryProvider<CardAttributeValueMapper>(
          create: (context) => CardAttributeValueMapper(),
        ),
        RepositoryProvider<CardAttributeMapper>(
          create: (context) => CardAttributeMapper(context.read(), context.read()),
        ),
        RepositoryProvider<CardSubtitleMapper>(
          create: (context) => CardSubtitleMapper(context.read()),
        ),
        RepositoryProvider<CardMapper>(
          create: (context) => CardMapper(context.read(), context.read()),
        ),
      ],
      child: child,
    );
  }
}
