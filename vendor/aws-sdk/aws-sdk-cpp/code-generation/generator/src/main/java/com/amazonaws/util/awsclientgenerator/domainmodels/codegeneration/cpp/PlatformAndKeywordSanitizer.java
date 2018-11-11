/*
* Copyright 2010-2017 Amazon.com, Inc. or its affiliates. All Rights Reserved.
*
* Licensed under the Apache License, Version 2.0 (the "License").
* You may not use this file except in compliance with the License.
* A copy of the License is located at
*
*  http://aws.amazon.com/apache2.0
*
* or in the "license" file accompanying this file. This file is distributed
* on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
* express or implied. See the License for the specific language governing
* permissions and limitations under the License.
*/

package com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp;

import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

public class PlatformAndKeywordSanitizer {
    private static final Set<String> FORBIDDEN_WORDS;

    private static final Map<Character,Character> ENUM_CHARS_MAPPING;

    static {
        Set<String> words = new HashSet<>();
        words.add("alignas");
        words.add("alignof");
        words.add("and");
        words.add("and_eq");
        words.add("asm");
        words.add("atomic_cancel");
        words.add("atomic_commit");
        words.add("atomic_noexcept");
        words.add("auto");
        words.add("bitand");
        words.add("bitor");
        words.add("bool");
        words.add("break");
        words.add("case");
        words.add("catch");
        words.add("char");
        words.add("char16_t");
        words.add("char32_t");
        words.add("class");
        words.add("compl");
        words.add("concept");
        words.add("const");
        words.add("constexpr");
        words.add("const_cast");
        words.add("continue");
        words.add("co_await");
        words.add("co_return");
        words.add("co_yeild");
        words.add("decltype");
        words.add("default");
        words.add("delete");
        words.add("do");
        words.add("double");
        words.add("dynamic_cast");
        words.add("else");
        words.add("enum");
        words.add("explicit");
        words.add("export");
        words.add("extern");
        words.add("false");
        words.add("float");
        words.add("for");
        words.add("friend");
        words.add("goto");
        words.add("if");
        words.add("import");
        words.add("inline");
        words.add("int");
        words.add("long");
        words.add("moduel");
        words.add("mutable");
        words.add("namespace");
        words.add("new");
        words.add("noexcept");
        words.add("not");
        words.add("not_eq");
        words.add("nullptr");
        words.add("operator");
        words.add("or");
        words.add("or_eq");
        words.add("private");
        words.add("protected");
        words.add("public");
        words.add("reflexpr");
        words.add("register");
        words.add("reinterpret_cast");
        words.add("requires");
        words.add("return");
        words.add("short");
        words.add("signed");
        words.add("sizeof");
        words.add("static");
        words.add("static_assert");
        words.add("static_cast");
        words.add("struct");
        words.add("switch");
        words.add("synchronized");
        words.add("template");
        words.add("this");
        words.add("thread_local");
        words.add("throw");
        words.add("true");
        words.add("try");
        words.add("typeof");
        words.add("typeid");
        words.add("typename");
        words.add("union");
        words.add("unsigned");
        words.add("using");
        words.add("virtual");
        words.add("void");
        words.add("volatile");
        words.add("wchar_t");
        words.add("while");
        words.add("xor");
        words.add("xor_eq");

        words.add("ANDROID");
        words.add("BOOL");
        words.add("CHAR");
        words.add("DEBUG");
        words.add("DELETE");
        words.add("Double");
        words.add("ERROR");
        words.add("GET");
        words.add("NEW");
        words.add("NULL");
        words.add("PRIVATE");
        words.add("PUBLIC");
        words.add("STATIC");
        words.add("T_CHAR");
        words.add("DOMAIN");
        words.add("*");
        //ok you get the idea... add them as you encounter them.
        FORBIDDEN_WORDS = Collections.unmodifiableSet(words);
    }

    static {
        Map<Character,Character> mapping = new HashMap<>();
        mapping.put('-', '_');
        mapping.put(':', '_');
        mapping.put('.', '_');
        mapping.put('*', '_');
        mapping.put('/', '_');
        mapping.put('(', '_');
        mapping.put(')', '_');
        mapping.put(' ', '_');
        ENUM_CHARS_MAPPING = Collections.unmodifiableMap(mapping);
    }

    // Converts C2J enum strings to a valid character set for c++.
    public static String fixEnumValue (String enumValue) {
        String enumMemberName = enumValue;

        for (Character invalid : ENUM_CHARS_MAPPING.keySet()) {
            enumMemberName = enumMemberName.replace(invalid, ENUM_CHARS_MAPPING.get(invalid));
        }

        enumMemberName = enumMemberName.replaceAll("_{2,}", "_").replaceAll("_$", "");

        if (FORBIDDEN_WORDS.contains(enumMemberName)) {
            enumMemberName += "_";
        }

        //replace starting number with underscore.
        char firstChar = enumMemberName.charAt(0);
        if(firstChar >= '0' && firstChar <= '9') {
            enumMemberName = "_" + enumMemberName;
        }

        return enumMemberName;
    }
}
