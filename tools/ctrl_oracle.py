#!/usr/bin/env python3
"""Compare the original boolean packet codec and per-player attachment update.

RegistryFriendlyByteBuf and ServerPlayer are small host substitutes; native
framing/authentication/ordering and key events require the game scenario.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import subprocess
import upstream
from controller_oracle import fragment
ROOT=upstream.ROOT
SOURCE=upstream.SOURCE/'src/main/java/appeng/core/network/serverbound/UpdateHoldingCtrlPacket.java'
WORK=ROOT/'target/ctrl-oracle'


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--java-home',type=Path,default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None)
    args=p.parse_args();upstream.verify(upstream.SOURCE)
    methods='\n'.join(fragment(SOURCE.read_text(),signature) for signature in [
        'public static UpdateHoldingCtrlPacket decode(', 'public void write(', 'public void handleOnServer('])
    java=r'''
import java.io.*;
import java.util.*;
public class CtrlOracle {
 static class RegistryFriendlyByteBuf {
  final DataInputStream in; final ByteArrayOutputStream out=new ByteArrayOutputStream();
  RegistryFriendlyByteBuf(byte[] data){in=new DataInputStream(new ByteArrayInputStream(data));}
  boolean readBoolean(){try{return in.readBoolean();}catch(IOException e){throw new IllegalArgumentException(e);}}
  void writeBoolean(boolean value){out.write(value?1:0);}
 }
 static class ServerPlayer {boolean down;void setData(Object key,boolean value){down=value;}}
 static class AEAttachmentTypes {static final Object HOLDING_CTRL=new Object();}
 record UpdateHoldingCtrlPacket(boolean keyDown) { @@METHODS@@ }
 public static void main(String[] args)throws Exception {
  var players=new ServerPlayer[]{new ServerPlayer(),new ServerPlayer(),new ServerPlayer(),new ServerPlayer()};
  var reader=new BufferedReader(new InputStreamReader(System.in));String line;
  while((line=reader.readLine())!=null){var s=new StringTokenizer(line);int sender=Integer.parseInt(s.nextToken());var data=new byte[s.countTokens()];for(int i=0;i<data.length;i++)data[i]=(byte)Integer.parseInt(s.nextToken());
   try {var packet=UpdateHoldingCtrlPacket.decode(new RegistryFriendlyByteBuf(data));packet.handleOnServer(players[sender]);var buf=new RegistryFriendlyByteBuf(new byte[0]);packet.write(buf);System.out.print(buf.out.toByteArray()[0]);}catch(IllegalArgumentException e){System.out.print("bad");}
   for(var player:players)System.out.print(" "+(player.down?1:0));System.out.println();
  }
 }
}
'''.replace('@@METHODS@@',methods)
    WORK.mkdir(parents=True,exist_ok=True);path=WORK/'CtrlOracle.java';path.write_text(java)
    binary=lambda name:str(args.java_home/'bin'/name) if args.java_home else name
    subprocess.run([binary('javac'),str(path)],check=True)
    cases=[(sender,[]) for sender in range(4)]+[(sender,[value]) for sender in range(4) for value in range(256)]
    rng=random.Random(0xAE2C712)
    cases += [(rng.randrange(4),[rng.randrange(256) for _ in range(rng.randrange(8))]) for _ in range(10000)]
    data=''.join(str(sender)+''.join(' '+str(b) for b in payload)+'\n' for sender,payload in cases).encode()
    ref=subprocess.run([binary('java'),'-cp',str(WORK),'CtrlOracle'],input=data,capture_output=True,check=True).stdout
    port=subprocess.run(['cargo','run','--locked','--quiet','--example','ctrl_oracle'],cwd=ROOT,input=data,capture_output=True,check=True).stdout
    (WORK/'input.txt').write_bytes(data);(WORK/'java.txt').write_bytes(ref);(WORK/'rust.txt').write_bytes(port)
    if ref!=port:
        for i,(a,b) in enumerate(zip(ref.splitlines(),port.splitlines())):
            if a!=b:raise AssertionError(f'{cases[i]} Java={a!r} Rust={b!r}')
        raise AssertionError('output length differs')
    report={'passed':True,'cases':len(cases),'revision':upstream.LOCK['revision'],
            'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
            'verbatim_fragment_sha256':hashlib.sha256(methods.encode()).hexdigest(),
            'port_sha256':hashlib.sha256((ROOT/'mixins/ae2/ctrl.rs').read_bytes()).hexdigest(),
            'input_sha256':hashlib.sha256(data).hexdigest(),'output_sha256':hashlib.sha256(ref).hexdigest(),
            'scope':'Verbatim UpdateHoldingCtrlPacket decode/write/handleOnServer; all byte values, empty/trailing input and isolated player attachments',
            'limits':['RegistryFriendlyByteBuf uses DataInputStream boolean semantics; ServerPlayer is a boolean attachment stub',
                      'Source methods consume the first boolean byte; complete NeoForge outer-payload trailing-byte validation is not exercised',
                      'Native framing, authenticated sender identity, tick ordering, configured key events, lifecycle and multiplayer require separate evidence']}
    (ROOT/'docs/ctrl-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))


if __name__=='__main__':main()
