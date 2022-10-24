import 'package:flutter/material.dart';

class ThemeScreen extends StatelessWidget {
  const ThemeScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Design system (draft)'),
      ),
      body: SafeArea(
        child: Column(
          children: [
            const DefaultTabController(
              length: 3,
              child: TabBar(
                tabs: [
                  Tab(text: 'One'),
                  Tab(text: 'Two'),
                  Tab(text: 'Three'),
                ],
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text("Headline 1", style: Theme.of(context).textTheme.headline1),
                  Text("Headline 2", style: Theme.of(context).textTheme.headline2),
                  Text("Headline 3", style: Theme.of(context).textTheme.headline3),
                  Text("Headline 4", style: Theme.of(context).textTheme.headline4),
                  Text("Subtitle 1", style: Theme.of(context).textTheme.subtitle1),
                  Text("Subtitle 2", style: Theme.of(context).textTheme.subtitle2),
                  Text("Body 1", style: Theme.of(context).textTheme.bodyText1),
                  Text("Body 2", style: Theme.of(context).textTheme.bodyText2),
                  Text("Button", style: Theme.of(context).textTheme.button),
                  Text("Caption", style: Theme.of(context).textTheme.caption),
                  Text("Overline", style: Theme.of(context).textTheme.overline),
                  const SizedBox(height: 16.0),
                  ElevatedButton(
                    onPressed: () => {},
                    style: Theme.of(context).elevatedButtonTheme.style,
                    child: const Text('Button'),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
      bottomNavigationBar: BottomNavigationBar(
        items: const [
          BottomNavigationBarItem(
            icon: Icon(Icons.credit_card),
            label: 'Kaarten',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.qr_code),
            label: 'QR',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.settings_outlined),
            label: 'Instellingen',
          ),
        ],
      ),
    );
  }
}
