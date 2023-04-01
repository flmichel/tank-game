#!/bin/bash
java -jar plantuml.jar -svg $(find . -type f -name "*.puml" ! -name "*configuration*" ! -name "*definition*")