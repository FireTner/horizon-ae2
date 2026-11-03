package ae2reference;

import com.google.gson.*;
import com.mojang.brigadier.StringReader;
import com.mojang.serialization.*;
import java.io.*;
import java.lang.reflect.*;
import java.util.*;

/** Executes the verified original 1.21.1 classes; no client/world entry point. */
public final class MinecraftAdventureReference {
    private static final Gson JSON = new Gson();
    private static Object blocks, registries;
    @SuppressWarnings("rawtypes") private static Codec codec;
    @SuppressWarnings("rawtypes") private static DynamicOps ops;
    private static final Map<String, Object> predicates = new HashMap<>();
    private static final Map<String, Object> entities = new HashMap<>();
    private static Class<?> cls(String name) throws Exception { return Class.forName(name); }
    private static Object field(String type, String name) throws Exception { return cls(type).getField(name).get(null); }
    private static Object call(Object target, String type, String name, Class<?>[] types, Object... args) throws Exception {
        var m=cls(type).getDeclaredMethod(name,types);m.setAccessible(true);return m.invoke(target,args);
    }
    private static Object id(String id) throws Exception { return call(null,"akr","a",new Class<?>[]{String.class},id); }
    @SuppressWarnings({"rawtypes","unchecked"}) private static void boot() throws Exception {
        call(null,"ab","a",new Class<?>[0]);call(null,"akt","a",new Class<?>[0]);
        blocks=field("lt","e");
        registries=call(null,"ka","a",new Class<?>[]{cls("jz")},field("lt","aA"));
        ops=(DynamicOps)call(null,"akp","a",new Class<?>[]{DynamicOps.class,cls("jo$a")},field("up","a"),registries);
        codec=(Codec)field("csc","a");
    }
    private static Object tag(String snbt) throws Exception {
        var parser=cls("uz").getConstructor(StringReader.class).newInstance(new StringReader(snbt));
        return call(parser,"uz","d",new Class<?>[0]);
    }
    private static Object block(String name) throws Exception {
        return call(blocks,"jz","a",new Class<?>[]{cls("akr")},id(name));
    }
    private static Object state(JsonObject input) throws Exception {
        Object block=block(input.get("block").getAsString());
        Object state=call(block,"dfy","o",new Class<?>[0]);
        if(input.has("state")) {
            Object definition=call(block,"dfy","l",new Class<?>[0]);
            for(var entry:input.getAsJsonObject("state").entrySet()) {
                Object property=call(definition,"dtd","a",new Class<?>[]{String.class},entry.getKey());
                if(property==null) throw new IllegalArgumentException("unknown property: "+entry.getKey());
                var value=(Optional<?>)call(property,"duf","b",new Class<?>[]{String.class},entry.getValue().getAsString());
                state=call(state,"dte","a",new Class<?>[]{cls("duf"),Comparable.class},property,value.orElseThrow());
            }
        }
        return state;
    }
    private static void tags(JsonObject input) throws Exception {
        if(!input.has("tags")) return;
        var tags=new HashMap<Object,List<Object>>();
        for(var entry:input.getAsJsonObject("tags").entrySet()) {
            Object key=call(null,"awu","a",new Class<?>[]{cls("akq"),cls("akr")},field("lu","f"),id(entry.getKey()));
            var holders=new ArrayList<Object>();
            for(var name:entry.getValue().getAsJsonArray()) {
                holders.add(((Optional<?>)call(blocks,"ju","c",new Class<?>[]{cls("akr")},id(name.getAsString()))).orElseThrow());
            }
            tags.put(key,holders);
        }
        call(blocks,"ju","a",new Class<?>[]{Map.class},tags);
    }
    private static JsonObject properties() throws Exception {
        var out=new JsonObject();
        for(Object block:(Iterable<?>)blocks) {
            var row=new JsonObject();
            var state=call(block,"dfy","o",new Class<?>[0]);
            var values=(Map<?,?>)call(state,"dte","C",new Class<?>[0]);
            for(Object property:values.keySet()) {
                var definition=new JsonObject();var ranks=new JsonObject();String kind=null;
                for(Object value:(Collection<?>)call(property,"duf","a",new Class<?>[0])) {
                    String name=(String)call(property,"duf","a",new Class<?>[]{Comparable.class},value);
                    int rank;
                    if(value instanceof Boolean b){kind="boolean";rank=b?1:0;}
                    else if(value instanceof Integer n){kind="integer";rank=n;}
                    else if(value instanceof Enum<?> e){kind="enum";rank=e.ordinal();}
                    else throw new IllegalArgumentException("unsupported property class "+value.getClass());
                    ranks.addProperty(name,rank);
                }
                definition.addProperty("kind",kind);definition.add("values",ranks);
                row.add((String)call(property,"duf","f",new Class<?>[0]),definition);
            }
            var name=call(blocks,"jz","b",new Class<?>[]{Object.class},block).toString();out.add(name,row);
        }
        return out;
    }
    private static JsonArray textView(String value) {
        var out=new JsonArray();for(char c:value.toCharArray())out.add((int)c);return out;
    }
    private static JsonArray view(Object value) throws Exception {
        int kind=((Number)call(value,"uy","b",new Class<?>[0])).intValue();
        var out=new JsonArray();out.add(kind);
        if(kind==10) {
            var data=new JsonArray();
            for(Object k:(Set<?>)call(value,"ub","e",new Class<?>[0])) {
                var pair=new JsonArray();pair.add(textView((String)k));pair.add(view(call(value,"ub","c",new Class<?>[]{String.class},k)));data.add(pair);
            }
            out.add(data);
        } else if(kind==7||kind==9||kind==11||kind==12) {
            var data=new JsonArray();for(Object v:(Iterable<?>)value)data.add(view(v));out.add(data);
        } else if(kind==8) out.add(textView((String)call(value,"uy","s_",new Class<?>[0])));
        else if(kind>=1&&kind<=6) {
            var n=(Number)call(value,"ur","l",new Class<?>[0]);
            if(kind==5)out.add(Long.toString(Integer.toUnsignedLong(Float.floatToRawIntBits(n.floatValue()))));
            else if(kind==6)out.add(Long.toUnsignedString(Double.doubleToRawLongBits(n.doubleValue())));
            else out.add(n.longValue());
        } else out.add(JsonNull.INSTANCE);
        return out;
    }
    private static void literals(Object value,JsonObject out)throws Exception {
        int kind=((Number)call(value,"uy","b",new Class<?>[0])).intValue();
        if(kind==8){String text=(String)call(value,"uy","s_",new Class<?>[0]);if(text.stripLeading().startsWith("{"))try{Object parsed=call(null,"uz","a",new Class<?>[]{String.class},text);if(((Number)call(parsed,"uy","b",new Class<?>[0])).intValue()==10)out.add(text,view(parsed));}catch(Exception ignored){}}
        else if(kind==10){for(Object k:(Set<?>)call(value,"ub","e",new Class<?>[0]))literals(call(value,"ub","c",new Class<?>[]{String.class},k),out);}
        else if(kind==9){for(Object v:(Iterable<?>)value)literals(v,out);}
    }
    @SuppressWarnings({"rawtypes","unchecked"}) private static JsonObject test(JsonObject input) throws Exception {
        tags(input);
        var out=new JsonObject();Object value;
        Object raw=tag(input.get("component").getAsString());out.add("component_view",view(raw));
        var literalPool=new JsonObject();literals(raw,literalPool);out.add("nbt_literals",literalPool);
        String cache=input.has("cache")?input.get("cache").getAsString():null;
        if(cache!=null && predicates.containsKey(cache)) value=predicates.get(cache);
        else {
            DataResult decoded=codec.parse(ops,raw);
            out.addProperty("decoded",decoded.result().isPresent());
            if(decoded.result().isEmpty()) return out;
            value=decoded.result().get();if(cache!=null)predicates.put(cache,value);
        }
        out.addProperty("decoded",true);
        out.addProperty("show",(Boolean)call(value,"csc","a",new Class<?>[0]));
        Object state=state(input);
        var stateView=new JsonObject();
        for(var entry:((Map<?,?>)call(state,"dte","C",new Class<?>[0])).entrySet()) {
            String name=(String)call(entry.getKey(),"duf","f",new Class<?>[0]);
            String text=(String)call(entry.getKey(),"duf","a",new Class<?>[]{Comparable.class},entry.getValue());stateView.addProperty(name,text);
        }
        out.add("state_view",stateView);
        JsonArray position=input.has("pos")?input.getAsJsonArray("pos"):JsonParser.parseString("[0,64,0]").getAsJsonArray();
        Object pos=cls("jd").getConstructor(int.class,int.class,int.class).newInstance(position.get(0).getAsInt(),position.get(1).getAsInt(),position.get(2).getAsInt());
        Object entity=null;
        if(input.has("entity")) {
            String key=input.has("entity_key")?input.get("entity_key").getAsString():null;
            Object data=tag(input.get("entity").getAsString());
            if(key!=null && entities.containsKey(key)) {
                entity=entities.get(key);call(entity,"dqh","c",new Class<?>[]{cls("ub"),cls("jo$a")},data,registries);
            } else {
                entity=call(null,"dqh","a",new Class<?>[]{cls("jd"),cls("dtc"),cls("ub"),cls("jo$a")},pos,state,data,registries);
                if(key!=null)entities.put(key,entity);
            }
        }
        final Object selectedState=state, selectedEntity=entity;
        Object world=Proxy.newProxyInstance(MinecraftAdventureReference.class.getClassLoader(),new Class<?>[]{cls("dcz")},(proxy,method,args)-> {
            if(method.getReturnType()==cls("dtc")) return selectedState;
            if(method.getReturnType()==cls("dqh")) return selectedEntity;
            if(method.getReturnType()==cls("ka")) return registries;
            if(method.getReturnType()==boolean.class) return true;
            if(method.getReturnType()==int.class) return 0;
            if(method.getName().equals("toString")) return "AE2 reference level";
            throw new UnsupportedOperationException(method.toString());
        });
        Object host=cls("dtg").getConstructor(cls("dcz"),cls("jd"),boolean.class).newInstance(world,pos,true);
        boolean result=(Boolean)call(value,"csc","a",new Class<?>[]{cls("dtg")},host);
        out.addProperty("matches",result);
        if(entity!=null) {Object saved=call(entity,"dqh","b",new Class<?>[]{cls("jo$a")},registries);out.addProperty("saved_entity",saved.toString());out.add("entity_view",view(saved));}
        return out;
    }
    public static void main(String[] args)throws Exception {
        var output=System.out;boot();
        if(args.length>0 && args[0].equals("--properties")) { output.println(JSON.toJson(properties()));return; }
        var reader=new BufferedReader(new InputStreamReader(System.in));String line;
        while((line=reader.readLine())!=null) {
            var input=JsonParser.parseString(line).getAsJsonObject();
            if(input.has("integer")) {
                var result=new JsonObject();String value;
                if(input.get("integer").isJsonArray()) {var chars=input.getAsJsonArray("integer");var text=new StringBuilder();for(var c:chars)text.append((char)c.getAsInt());value=text.toString();}
                else value=input.get("integer").getAsString();
                try{result.addProperty("integer",Integer.valueOf(value));}catch(NumberFormatException e){result.add("integer",JsonNull.INSTANCE);}
                output.println(JSON.toJson(result));
            } else if(input.has("parse_snbt")) {
                var result=new JsonObject();
                try {result.add("parse_snbt",view(call(null,"uz","a",new Class<?>[]{String.class},input.get("parse_snbt").getAsString())));}catch(Exception e){result.add("parse_snbt",JsonNull.INSTANCE);}
                output.println(JSON.toJson(result));
            } else if(input.has("compare_nbt")) {
                var pair=input.getAsJsonArray("compare_nbt");var expected=tag(pair.get(0).getAsString());var actual=tag(pair.get(1).getAsString());
                var result=new JsonObject();result.add("expected",view(expected));result.add("actual",view(actual));
                result.addProperty("compare_nbt",(Boolean)call(null,"uq","a",new Class<?>[]{cls("uy"),cls("uy"),boolean.class},expected,actual,true));output.println(JSON.toJson(result));
            } else output.println(JSON.toJson(test(input)));
        }
    }
}
