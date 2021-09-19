import React, { useRef, useEffect, useState } from "react";
import { Table, Button, TextInput, Pane, majorScale } from "evergreen-ui";

export function Logs(props) {
  const profiles = [
    { name: "rasmus", lastActivity: "2022-01-01", ltv: 123 },
    { name: "viitanen", lastActivity: "2022-01-02", ltv: 456 },
  ];

  return (
    <Table>
      <Table.Head>
        <Table.SearchHeaderCell />
        <Table.TextHeaderCell>Last Activity</Table.TextHeaderCell>
        <Table.TextHeaderCell>ltv</Table.TextHeaderCell>
      </Table.Head>
      <Table.Body height={240}>
        {profiles.map((profile) => (
          <Table.Row
            key={profile.id}
            height={32}
            isSelectable
            onSelect={() => alert(profile.name)}
          >
            <Table.TextCell>{profile.name}</Table.TextCell>
            <Table.TextCell>{profile.lastActivity}</Table.TextCell>
            <Table.TextCell isNumber>{profile.ltv}</Table.TextCell>
          </Table.Row>
        ))}
      </Table.Body>
    </Table>
  );
}
