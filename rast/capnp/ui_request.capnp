@0xd929be78b46eabb8;

struct UiRequest {
  uiMessage :union {
    ping @0 :Void;
    getIps @1 :Void;
    getIpData @2 :Text;
    command @3 :Text;
  }
}

struct UiResponse {
  uiMessage :union {
    pong @0 :Void;
    getIps @1 :List(Text);
    getIpData :group {
      ip @2: Text;
    }
    command @3 :Text;
  }
}

