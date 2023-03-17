import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/theme/wallet_theme.dart';

/// Widget that is to be used to wrap pumped test widgets.
/// Makes sure the theme, translations and MediaQuery is provided.
class WalletAppTestWidget extends StatelessWidget {
  final Widget child;

  const WalletAppTestWidget({required this.child, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: WalletTheme.light,
      darkTheme: WalletTheme.dark,
      localizationsDelegates: const [
        AppLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: const [
        Locale('en', ''), // English, no country code
        Locale('nl', ''), // Dutch, no country code
      ],
      home: Material(child: child),
    );
  }
}

WidgetWrapper walletAppWrapper({List<BlocProvider>? providers}) {
  return (child) {
    if (providers == null) return WalletAppTestWidget(child: child);
    return MultiBlocProvider(
      providers: providers,
      child: WalletAppTestWidget(child: child),
    );
  };
}

extension TestWidgetExtensions on Widget {
  /// Wraps the widget with a BlocProvider to provide the [bloc]
  /// and configures it to emit the provided [state].
  ///
  /// Useful to configure the state of a screen for a UI test.
  Widget withState<B extends BlocBase<S>, S>(B bloc, S state) => BlocProvider<B>(
        create: (c) {
          assert(bloc is MockBloc, 'Can only provide mocked state on MockBloc');
          whenListen(
            bloc,
            Stream<S>.empty(),
            initialState: state,
          );
          return bloc;
        },
        child: this,
      );
}
