import 'package:flutter/material.dart';

import '../../../wallet_routes.dart';

class PlaceholderScreen extends StatelessWidget {
  final String title;

  const PlaceholderScreen({required this.title, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(title),
      ),
      body: SafeArea(
        child: Center(
          child: Text(
            'Placeholder; $title',
          ),
        ),
      ),
    );
  }

  static void show(BuildContext context, String title, {bool secured = true}) {
    Navigator.push(
      context,
      secured
          ? SecuredPageRoute(builder: (c) => PlaceholderScreen(title: title))
          : MaterialPageRoute(builder: (c) => PlaceholderScreen(title: title)),
    );
  }
}
