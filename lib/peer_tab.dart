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

class _PeerTabState extends State<PeerTab> with AutomaticKeepAliveClientMixin<PeerTab> {
  Adbflib _adbflib;
  _PeerTabState(this._adbflib);

  String _peerId = '';
  bool _searchingPeers = false;
  String _ownIdString = '';
  String _peerData;

  @override
  void initState() {
    super.initState();
    final int ownIntId = _adbflib.getOwnPeerId();
    _ownIdString = i64AsU64ToString(ownIntId);
    //
    this._startNetMessaging();
  }


  @override
  bool get wantKeepAlive => true;
  @override
  Widget build(BuildContext context) {
    return Container(
      child: Scaffold(
        body: Align(
          alignment: Alignment.topCenter,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.start,
            children: [
              const SizedBox(height: 20),
              Text(
                "Own Peer ID: $_ownIdString",
                style: TextStyle(
                  fontFamily: "monospace",
                  color: Colors.white,
                ),
              ),
              Divider(
                height: 50,
                thickness: 2,
                color: Colors.white,
              ),
              RaisedButton(
                color: _searchingPeers ? Colors.greenAccent : Colors.lime,
                child: Text(
                  'Start Peer Search',
                  style: TextStyle(
                    color: Colors.white,
                  ),
                ),
                onPressed: () {
                  if (!_searchingPeers) {
                    _findNewPeer();
                  }
                },
              ),
              Text(
                'First peer found:',
              ),
              const SizedBox(height: 5),
              Text(
                '$_peerId',
                style: TextStyle(
                  fontFamily: "monospace",
                  color: Colors.white,
                ),
              ),
              Divider(
                height: 60,
                thickness: 4,
                color: Colors.white,
              ),
              Text(
                ': $_peerData',
                style: TextStyle(
                  color: Colors.white,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _findNewPeer() async {
    _searchingPeers = true;
    setState(() {});
    final int peerInt = await _adbflib.findNewPeer();
    _peerId = i64AsU64ToString(peerInt);
    _searchingPeers = false;
    setState(() {});
  }

  void _startNetMessaging() async {
    while (true) {
      _peerData = await _adbflib.getNetUiMessages();
      setState(() {});
    }
  }


  // please ... this is a complete hack about
  // dart's non capability for u64, and some strange behavior with
  // numbers!!!
  String i64AsU64ToString(int number) {
    String out = '';
    if (number > 0) {
      out = number.toRadixString(16);
    } else {
      final lastHexDigit = (number % 16);
      number = -((number >> 4)^0xfffffffffffffff) -1;
      out = (number.toRadixString(16)) + ((lastHexDigit).toRadixString(16)) ;
    }
    return out;
  }
}