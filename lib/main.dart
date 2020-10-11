import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';

import 'search_tab.dart';
import 'peer_tab.dart';

void main() => runApp(MyApp());

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'adbfflutter',
      theme: ThemeData(
        primarySwatch: Colors.blue,
        brightness: Brightness.dark,
      ),
      home: DefaultTabController(
        length: 2,
        child: Scaffold(
          appBar: AppBar(
            bottom: TabBar(
              tabs: [
                Tab(
                    child: Align(
                      alignment: Alignment.center,
                      child: Text("Search"),
                    )
                ),
                Tab(
                    child: Align(
                      alignment: Alignment.center,
                      child: Text("Network"),
                    )
                )
              ],
            ),
            title: Text('adfbfflutter'),
          ),
          body: TabBarView(
            children: [
              SearchTab(),
              PeerTab(),
            ],
          ),
        ),
      ),
    );
  }
}
