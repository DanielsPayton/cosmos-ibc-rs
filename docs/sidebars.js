// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {

  learnSidebar: [
    {
      type: "category",
      label: "Welcome!",
      collapsed: true,
      items: [ { type: "autogenerated", dirName: 'learn/intro' } ],
    },
    {
      type: "category",
      label: "Overview",
      collapsed: true,
      items: [ { type: "autogenerated", dirName: 'learn/overview' } ],
    },
    {
      type: "category",
      label: "IBC Light Clients",
      collapsed: true,
      items: [ { type: "autogenerated", dirName: 'learn/clients' } ],
    },
    {
      type: "category",
      label: "IBC Applications",
      collapsed: true,
      items: [ { type: "autogenerated", dirName: 'learn/apps' } ],
    },
    {
      type: "category",
      label: "Resources",
      collapsed: false,
      items: [
        {
          type: "link",
          label: "IBC Protocol",
          href: "https://github.com/cosmos/ibc/",
        },
        {
          type: "link",
          label: "IBC Data Structures",
          href: "https://github.com/cosmos/ibc-proto-rs/",
        },
        {
          type: "link",
          label: "Basecoin",
          href: "https://github.com/informalsystems/basecoin-rs",
        },
        {
          type: "link",
          label: "Project Board",
          href: "https://github.com/orgs/cosmos/projects/27",
        },
        {
          type: "link",
          label: "Road to V1",
          href: "https://github.com/cosmos/ibc-rs/issues/554",
        },
      ],
    },
  ],
  developersSidebar: [
    {
      type: "category",
      label: "Getting Started",
      collapsed: true,
      items: [ { type: "autogenerated", dirName: 'developers/intro' } ],
    },
    {
      type: "category",
      label: "ADRs",
      collapsed: true,
      items: [ { type: "autogenerated", dirName: 'developers/architecture' } ],
    },
    {
      type: "category",
      label: "Developer Guide",
      collapsed: false,
      items: [ { type: "autogenerated", dirName: 'developers/build' } ],
    },
    {
      type: "category",
      label: "Integration",
      collapsed: true,
      items: [ { type: "autogenerated", dirName: 'developers/integration' } ],
    },
    {
      type: "category",
      label: "IBC Stack Upgrades",
      collapsed: true,
      items: [ { type: "autogenerated", dirName: 'developers/upgrades' } ],
    },
    {
      type: "category",
      label: "Migrations",
      collapsed: true,
      items: [ { type: "autogenerated", dirName: 'developers/migrations' } ],
    },
  ],
};

module.exports = sidebars;
