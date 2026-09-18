//! Public market, event, calendar, security, and price contracts.

mod discovery;
mod events;
mod market;

pub use market::{
	AgaraEngineConfig, BaseEventDetailCategories, BaseEventDetailCategoriesPath,
	BaseEventDetailPrimaryCategory, BaseEventListItemPrimaryCategory, BaseMarketDisplay,
	BaseOutcomeDisplay, CategorySection, CategorySectionMeta, GridColumnsHint, MainMarketGroup,
	MarketDetail, MarketGroup, MarketLpIncentive, MarketOutcome, MarketResolution, MarketTab,
	MarketsListEventRef, MarketsListItem, MarketsListResponse, PolymarketMarketDisplay,
	ResolutionSourceLink,
};

pub use events::{
	BaseEventDetail, BaseEventListItem, EventDetail, EventDisplay, EventDisplayTopCandidates,
	EventListItem, EventListSection, EventListSectionCategory, GameState, GameStateMatch,
	GameStateScore, GameStateScoreMatch, GameStateScoreSets, GameStateScoreSetsTiebreak,
	ListingMarket, ListingMarketOutcome, PolymarketEventDisplay, PolymarketEventDisplaySeries,
	SportsTeam,
};

pub use discovery::{
	CalendarRangeQuery, CategoryResource, CategoryResourceEventTypes, EventCanonicalUrlResponse,
	EventsListResponse, EventsQuery, MarketsQuery, NextSessionQuery, NextSessionResponse,
	PnlHistoryQuery, PriceHistory, PricePoint, PricePointQuery, PriceTicksQuery,
	SearchCategoryItem, SearchCategoryRef, SearchEventItem, SearchMarketEventRef, SearchMarketItem,
	SearchOutcome, SearchQuery, SearchRelevance, SearchResponse, SecuritiesResponse, Security,
	SecurityDetail, SecurityProvider, SessionSegment, StringOrNumber, TokenHistoryQuery,
	TradingDay, TradingDaysResponse, TradingVenue, TradingVenuesResponse,
};
