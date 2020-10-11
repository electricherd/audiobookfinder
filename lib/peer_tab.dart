// peer_tab.dart
import 'package:adbflib/adbflib.dart';
import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';


class PeerTab extends StatefulWidget {
  Adbflib adbflib;
  PeerTab(this.adbflib, {Key key}) : super(key: key);
  @override
  _PeerTabState createState() => _PeerTabState(adbflib);
}

class _PeerTabState extends State<PeerTab> {
  Adbflib adbflib;
  _PeerTabState(this.adbflib);

  String _peer_id = '';
  bool _searching_peers = false;

  @override
  Widget build(BuildContext context) {
    return Container(
      child: Scaffold(
        body: Align(
          alignment: Alignment.topCenter,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.start,
            children: [
              const SizedBox(height: 50),
              RaisedButton(
                color: _searching_peers ? Colors.greenAccent : Colors.lime,
                child: Text(
                  'Start Peer Search',
                  style: TextStyle(
                    color: Colors.white,
                  ),
                ),
                onPressed: () {
                  if (!_searching_peers) {
                    _findNewPeer();
                  }
                },
              ),
              Text(
                'First peer found:',
              ),
              const SizedBox(height: 5),
              Text(
                '$_peer_id',
                style: TextStyle(
                  fontFamily: "monospace",
                  color: Colors.white,
                ),
              )
            ],
          ),
        ),
      ),
    );
  }

  void _findNewPeer() async {
    _searching_peers = true;
    setState(() {});
    int peer_int = await adbflib.findNewPeer();
    // it's int not uint
    if (peer_int < 0) {
      peer_int = -peer_int;
    }
    _peer_id = peer_int.toRadixString(16);
    _searching_peers = false;
    setState(() {});
  }

}