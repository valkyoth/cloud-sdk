use super::*;
use crate::identifiers::*;

// Exact OpenAPI query-name inventory; checked against fetched source by the
// request-policy gate, independently of the runtime operation match arms.
const CONTRACTS: &[(QueryOperation, &str)] = &[
    (QueryOperation::Categories, "sort page per_page seek"),
    (QueryOperation::Keywords, "sort page per_page seek"),
    (
        QueryOperation::Crates,
        "sort q include_yanked category all_keywords keyword letter user_id team_id following ids[] page per_page seek",
    ),
    (QueryOperation::Crate, "include"),
    (QueryOperation::Downloads, "include"),
    (
        QueryOperation::Versions,
        "include sort nums[] page per_page seek",
    ),
    (QueryOperation::ReverseDependencies, "page per_page seek"),
    (QueryOperation::VersionDownloads, "before_date"),
    (QueryOperation::User, "include"),
    (
        QueryOperation::GithubConfigs,
        "crate user_id page per_page seek",
    ),
    (
        QueryOperation::GitlabConfigs,
        "crate user_id page per_page seek",
    ),
];

fn valid<T, E>(v: Result<T, E>) -> T {
    v.unwrap_or_else(|_| unreachable!("contract fixture"))
}

#[test]
fn every_parameter_kind_obeys_the_independent_operation_matrix() {
    let name = valid(CrateName::new("serde"));
    let keyword = valid(Keyword::new("parser"));
    let version = valid(Version::new("1.0.0"));
    let id = valid(NumericId::new(1));
    let inputs = [
        Parameter::Search(valid(SearchQuery::new("parser"))),
        Parameter::Category(valid(CategorySlug::new("parsing"))),
        Parameter::Keyword(keyword),
        Parameter::AllKeywords(&[keyword]),
        Parameter::Letter('a'),
        Parameter::UserId(id),
        Parameter::TeamId(id),
        Parameter::Following,
        Parameter::Ids(&[name]),
        Parameter::Versions(&[version]),
        Parameter::IncludeYanked,
        Parameter::Crate(name),
        Parameter::Page(valid(Page::new(1))),
        Parameter::PerPage(valid(PerPage::new(10))),
        Parameter::Seek(valid(Seek::new("OTg"))),
        Parameter::BeforeDate(valid(Date::new("2024-01-01"))),
    ];
    for (operation, keys) in CONTRACTS {
        for input in inputs {
            assert_eq!(
                Query::new(*operation, &[input]).is_ok(),
                keys.split(' ').any(|key| key == input.key()),
                "{operation:?}, {input:?}"
            );
        }
    }
}

#[test]
fn all_sort_and_include_choices_are_operation_scoped() {
    let sorts = [
        Sort::Alpha,
        Sort::Crates,
        Sort::Alphabetical,
        Sort::Relevance,
        Sort::Downloads,
        Sort::RecentDownloads,
        Sort::RecentUpdates,
        Sort::New,
        Sort::Date,
        Sort::Semver,
    ];
    let includes = [
        Include::Versions,
        Include::Keywords,
        Include::Categories,
        Include::Badges,
        Include::Downloads,
        Include::DefaultVersion,
        Include::Full,
        Include::ReleaseTracks,
        Include::LinkedAccounts,
    ];
    for (op, _) in CONTRACTS {
        let allowed_sorts = match op {
            QueryOperation::Categories | QueryOperation::Keywords => "alpha crates",
            QueryOperation::Crates => {
                "alphabetical relevance downloads recent-downloads recent-updates new"
            }
            QueryOperation::Versions => "date semver",
            _ => "",
        };
        for sort in sorts {
            assert_eq!(
                Query::new(*op, &[Parameter::Sort(sort)]).is_ok(),
                allowed_sorts.split(' ').any(|v| v == sort.as_str())
            );
            assert_eq!(valid(Sort::parse(sort.as_str())), sort);
        }
        let allowed_includes = match op {
            QueryOperation::Crate => {
                "versions keywords categories badges downloads default_version full"
            }
            QueryOperation::Downloads => "versions",
            QueryOperation::Versions => "release_tracks",
            QueryOperation::User => "linked_accounts",
            _ => "",
        };
        for include in includes {
            let values = [include];
            assert_eq!(
                Query::new(*op, &[Parameter::Include(valid(IncludeSet::new(&values)))]).is_ok(),
                allowed_includes.split(' ').any(|v| v == include.as_str())
            );
            assert_eq!(valid(Include::parse(include.as_str())), include);
        }
    }
}
