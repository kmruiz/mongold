((block_comment)* @javadoc
  (field_declaration
    type: (generic_type (type_identifier) (type_arguments (type_identifier) @document)) @type
    declarator: (variable_declarator) @field
    (#match? @type "Collection")
    (#match? @javadoc "@mongodb.namespace")))