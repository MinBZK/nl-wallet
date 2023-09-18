import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../util/mapper/card/card_attribute_label_mapper.dart';
import '../util/mapper/card/card_attribute_mapper.dart';
import '../util/mapper/card/card_mapper.dart';
import '../util/mapper/pid/pid_attributes_mapper.dart';
import '../util/mapper/pid/pid_core_attributes_mapper.dart';
import '../util/mapper/pid/pid_data_attributes_mapper.dart';
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
          create: (context) =>
              (provideMocks ? PidDataAttributeMapper() : PidCoreAttributeMapper()) as PidAttributeMapper,
        ),
        RepositoryProvider<CardAttributeLabelMapper>(
          create: (context) => CardAttributeLabelMapper(),
        ),
        RepositoryProvider<CardAttributeMapper>(
          create: (context) => CardAttributeMapper(context.read()),
        ),
        RepositoryProvider<CardMapper>(
          create: (context) => CardMapper(context.read()),
        ),
      ],
      child: child,
    );
  }
}
