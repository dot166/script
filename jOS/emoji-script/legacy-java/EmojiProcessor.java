package io.github.dot166.emojiscript;

import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.File;
import java.io.FileReader;
import java.io.FileWriter;
import java.io.IOException;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

public class EmojiProcessor {
    public static void main(String[] args) {
        if (args.length < 1 || args.length > 3) {
            System.out.println("Usage: emoji [url|https://unicode.org/Public/emoji/16.0/emoji-test.txt] -F(Force fallback) -v(Verbose)");
            System.exit(1);
        }

        String url = args[0];
        StringBuilder response = new StringBuilder();
        try {
            URL obj = new URL(url);
            HttpURLConnection con = (HttpURLConnection) obj.openConnection();
            con.setRequestMethod("GET");
            BufferedReader in = new BufferedReader(new InputStreamReader(con.getInputStream()));
            String inputLine;
            while ((inputLine = in.readLine()) != null) {
                response.append(inputLine).append("\n");
            }
            in.close();
        } catch (IOException e) {
            e.printStackTrace();
            return;
        }

        String groupName = "";
        Map<String, List<String>> items = new HashMap<>();

        for (String line : response.toString().split("\n")) {
            if (line.startsWith("# subgroup: ")) {
                groupName = line.split("\\s+", 3)[2];
            } else if (line.contains("; fully-qualified") && !line.contains("skin tone")) {
                String item = line.split(";")[0].trim().replace(" ", ",");
                items.computeIfAbsent(groupName, k -> new ArrayList<>()).add(item);
            }
        }

        String absolutePath = new File("").getAbsolutePath();
        String relativePath = "../../../platform_packages_inputmethods_LatinIME/java/res/values-v19/emoji-categories.xml";
        Path targetPath = Paths.get(absolutePath, relativePath).toAbsolutePath();

        try {
            StringBuilder lines = new StringBuilder();
            File inputFile = targetPath.toFile();
            boolean useFallback = (!targetPath.toFile().exists() || contains(args, "-F"));
            if (useFallback) {
                inputFile = Paths.get(absolutePath, "../fallback.xml").toAbsolutePath().toFile();
            }
            BufferedReader reader = new BufferedReader(new FileReader(inputFile));
            String line;
            while ((line = reader.readLine()) != null) {
                lines.append(line).append("\n");
            }
            reader.close();

            String content = lines.toString();
            for (String key : items.keySet()) {
                String header = "<!-- " + key + " -->";
                int start = content.indexOf(header);

                while (start != -1) {
                    int end1 = content.indexOf("</array>", start);
                    int end2 = content.indexOf("<!--", start + 1);

                    int minEnd = Math.min(end1 == -1 ? Integer.MAX_VALUE : end1, end2 == -1 ? Integer.MAX_VALUE : end2);
                    String replace = content.substring(start, minEnd).trim();

                    StringBuilder built = new StringBuilder(header + "\n");
                    for (String c : items.get(key)) {
                        built.append("        <item>").append(c).append("</item>\n");
                    }
                    built = new StringBuilder(built.toString().trim());

                    content = content.replace(replace, built.toString());
                    start = content.indexOf(header, start + built.length());
                }
                //items.remove(key);
            }

            System.out.println("Updating emoji-categories.xml in LatinIME");
            if (contains(args, "-v")) {
                System.out.println(content);
            }
            if (targetPath.toFile().exists() && useFallback) {
                targetPath.toFile().delete();
            }
            BufferedWriter writer = new BufferedWriter(new FileWriter(targetPath.toFile()));
            writer.write(content);
            writer.close();
            System.out.println("Done!");

            System.out.println("Updating fallback.xml");
            BufferedWriter writer1 = new BufferedWriter(new FileWriter(Paths.get(absolutePath, "fallback.xml").toAbsolutePath().toFile()));
            writer1.write(content);
            writer1.close();
            System.out.println("Done!");
        } catch (IOException e) {
            e.printStackTrace();
        }

//        if (!items.isEmpty()) {
//            System.out.println("Could not process the following items automatically:");
//            for (String key : items.keySet()) {
//                StringBuilder built = new StringBuilder("        <!-- " + key + " -->");
//                for (String c : items.get(key)) {
//                    built.append("\n        <item>").append(c).append("</item>");
//                }
//                System.out.println(built);
//            }
//        }
    }


    /**
     * Checks that value is present as at least one of the elements of the array.
     * @param array the array to check in
     * @param value the value to check for
     * @return true if the value is present in the array
     */
    public static boolean contains(String[] array, String value) {
        return indexOf(array, value) != -1;
    }

    /**
     * Return first index of {@code value} in {@code array}, or {@code -1} if
     * not found.
     */
    public static int indexOf(String[] array, String value) {
        if (array == null) return -1;
        for (int i = 0; i < array.length; i++) {
            if (Objects.equals(array[i], value)) return i;
        }
        return -1;
    }
}

